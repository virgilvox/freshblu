/// FreshBlu CLI - meshblu-util compatible command line interface
///
/// Usage examples:
///   freshblu register                    # Register a new device
///   freshblu register -d '{"type":"sensor"}' # Register with properties
///   freshblu whoami                      # Show authenticated device
///   freshblu get <uuid>                  # Get a device
///   freshblu update <uuid> -d '{"color":"red"}'
///   freshblu message -d '{"devices":["*"],"payload":"hello"}'
///   freshblu subscribe <emitter-uuid> broadcast.sent
///   freshblu token generate <uuid>
///   freshblu server                      # Start the server
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde_json::Value;
#[cfg(feature = "server")]
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "freshblu",
    about = "FreshBlu CLI - Meshblu-compatible IoT device registry & messaging",
    version
)]
struct Cli {
    /// FreshBlu server URL
    #[arg(
        short = 'S',
        long,
        env = "FRESHBLU_SERVER",
        default_value = "http://localhost:3000"
    )]
    server: String,

    /// Device UUID for auth
    #[arg(short = 'U', long, env = "FRESHBLU_UUID")]
    uuid: Option<String>,

    /// Device token for auth
    #[arg(short = 'T', long, env = "FRESHBLU_TOKEN")]
    token: Option<String>,

    /// Path to config file (default: ./freshblu.json)
    #[arg(short = 'c', long)]
    config: Option<PathBuf>,

    /// Output format (json|pretty|plain)
    #[arg(short = 'f', long, default_value = "pretty")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FreshBlu server
    Server {
        #[arg(long, env = "FRESHBLU_HTTP_PORT", default_value = "3000")]
        port: u16,
        #[arg(long, env = "DATABASE_URL", default_value = "sqlite:freshblu.db")]
        db: String,
    },

    /// Register a new device
    Register {
        /// Device properties as JSON
        #[arg(short = 'd', long, default_value = "{}")]
        data: String,
        /// Device type
        #[arg(short = 't', long)]
        r#type: Option<String>,
        /// Save credentials to file
        #[arg(long, default_value = "freshblu.json")]
        save: String,
    },

    /// Get authenticated device info (whoami)
    Whoami,

    /// Get a device by UUID
    Get {
        uuid: String,
        /// Act as another device
        #[arg(long)]
        r#as: Option<String>,
    },

    /// Update a device
    Update {
        uuid: Option<String>,
        /// Properties to update as JSON
        #[arg(short = 'd', long)]
        data: String,
    },

    /// Unregister a device
    Unregister { uuid: Option<String> },

    /// Search for devices
    Search {
        /// Query as JSON
        #[arg(short = 'd', long, default_value = "{}")]
        query: String,
    },

    /// Send a message
    Message {
        /// Message as JSON (must include devices field)
        #[arg(short = 'd', long)]
        data: String,
    },

    /// Subscribe to device events
    Subscribe {
        emitter_uuid: String,
        /// Subscription type (broadcast.sent, message.received, etc.)
        subscription_type: String,
    },

    /// Token management
    Token {
        #[command(subcommand)]
        cmd: TokenCommands,
    },

    /// Check server status
    Status,

    /// Show current config
    Config,
}

#[derive(Subcommand)]
enum TokenCommands {
    /// Generate a new token for a device
    Generate {
        uuid: Option<String>,
        #[arg(long)]
        expires_on: Option<i64>,
        #[arg(long)]
        tag: Option<String>,
    },
    /// Revoke a token
    Revoke { uuid: Option<String>, token: String },
}

/// Saved credentials file (~/.freshblu.json or ./freshblu.json)
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Credentials {
    uuid: Option<String>,
    token: Option<String>,
    server: Option<String>,
}

impl Credentials {
    fn load(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

fn print_value(v: &Value, format: &str) {
    match format {
        "json" => println!("{}", v),
        "pretty" => println!("{}", serde_json::to_string_pretty(v).unwrap_or_default()),
        _ => println!("{}", v),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load credentials from file
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from("freshblu.json"));

    let mut creds = Credentials::load(&config_path);

    // CLI args override file
    if let Some(u) = &cli.uuid {
        creds.uuid = Some(u.clone());
    }
    if let Some(t) = &cli.token {
        creds.token = Some(t.clone());
    }
    creds.server = Some(cli.server.clone());

    let client = reqwest::Client::new();
    let server = cli.server.trim_end_matches('/');

    // Build auth header
    let auth_header = || -> Option<String> {
        let uuid = creds.uuid.as_ref()?;
        let token = creds.token.as_ref()?;
        let creds_str = format!("{}:{}", uuid, token);
        Some(format!("Basic {}", base64_encode(creds_str.as_bytes())))
    };

    match cli.command {
        Commands::Server { port, db } => {
            #[cfg(feature = "server")]
            {
                use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

                tracing_subscriber::registry()
                    .with(
                        EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| "freshblu=info,tower_http=debug".into()),
                    )
                    .with(tracing_subscriber::fmt::layer())
                    .init();

                let store: freshblu_store::DynStore = Arc::new(
                    freshblu_store::sqlite::SqliteStore::new(&db).await?,
                );

                let bus: freshblu_server::bus::DynBus =
                    Arc::new(freshblu_server::local_bus::LocalBus::new());

                let config = freshblu_server::ServerConfig {
                    http_port: port,
                    database_url: db.clone(),
                    ..Default::default()
                };

                let rate_limiter =
                    freshblu_server::RateLimiter::new(config.rate_limit, config.rate_window);
                let webhook_executor = Arc::new(freshblu_server::WebhookExecutor::new(
                    store.clone(),
                    bus.clone(),
                ));

                let state = freshblu_server::AppState {
                    store,
                    bus,
                    config,
                    rate_limiter,
                    webhook_executor,
                };

                let router = freshblu_server::build_router(state);
                let addr = format!("0.0.0.0:{}", port);
                let listener = tokio::net::TcpListener::bind(&addr).await?;

                println!(
                    "{}",
                    r#"
 _____              _     ____  _
|  ___| __ ___  ___| |__ | __ )| |_   _
| |_ | '__/ _ \/ __| '_ \|  _ \| | | | |
|  _|| | |  __/\__ \ | | | |_) | | |_| |
|_|  |_|  \___||___/_| |_|____/|_|\__,_|
"#
                    .cyan()
                );
                println!("  {} http://{}", "HTTP".green().bold(), addr);
                println!("  {} ws://localhost:{}/ws", "WS".green().bold(), port);
                println!("  {} {}", "DB".green().bold(), db);
                println!();

                axum::serve(listener, router).await?;
            }
            #[cfg(not(feature = "server"))]
            {
                let _ = (port, db);
                eprintln!(
                    "{}: The server feature is not compiled in.",
                    "error".red().bold()
                );
                eprintln!("Rebuild with: cargo install freshblu-cli --features server");
                std::process::exit(1);
            }
        }

        Commands::Register { data, r#type, save } => {
            let mut body: Value =
                serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}));

            if let Some(t) = r#type {
                body["type"] = Value::String(t);
            }

            let resp = client
                .post(format!("{}/devices", server))
                .json(&body)
                .send()
                .await?;

            let result: Value = resp.json().await?;

            // Save credentials
            let save_path = PathBuf::from(&save);
            let new_creds = Credentials {
                uuid: result
                    .get("uuid")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                token: result
                    .get("token")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                server: Some(server.to_string()),
            };
            new_creds.save(&save_path)?;

            eprintln!("{} Credentials saved to {}", "✓".green(), save);
            print_value(&result, &cli.format);
        }

        Commands::Whoami => {
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let resp = client
                .get(format!("{}/whoami", server))
                .header("Authorization", auth)
                .send()
                .await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Get { uuid, r#as } => {
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let mut req = client
                .get(format!("{}/devices/{}", server, uuid))
                .header("Authorization", auth);
            if let Some(as_uuid) = r#as {
                req = req.header("x-meshblu-as", as_uuid);
            }
            let resp = req.send().await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Update { uuid, data } => {
            let target_uuid = uuid
                .or_else(|| creds.uuid.clone())
                .ok_or_else(|| anyhow::anyhow!("UUID required"))?;
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let body: Value = serde_json::from_str(&data)?;
            let resp = client
                .put(format!("{}/devices/{}", server, target_uuid))
                .header("Authorization", auth)
                .json(&body)
                .send()
                .await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Unregister { uuid } => {
            let target_uuid = uuid
                .or_else(|| creds.uuid.clone())
                .ok_or_else(|| anyhow::anyhow!("UUID required"))?;
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let resp = client
                .delete(format!("{}/devices/{}", server, target_uuid))
                .header("Authorization", auth)
                .send()
                .await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Search { query } => {
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let body: Value = serde_json::from_str(&query)?;
            let resp = client
                .post(format!("{}/devices/search", server))
                .header("Authorization", auth)
                .json(&body)
                .send()
                .await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Message { data } => {
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let body: Value = serde_json::from_str(&data)?;
            let resp = client
                .post(format!("{}/messages", server))
                .header("Authorization", auth)
                .json(&body)
                .send()
                .await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Subscribe {
            emitter_uuid,
            subscription_type,
        } => {
            let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
            let subscriber_uuid = creds
                .uuid
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No UUID set"))?;
            let body = serde_json::json!({
                "emitterUuid": emitter_uuid,
                "subscriberUuid": subscriber_uuid,
                "type": subscription_type
            });
            let resp = client
                .post(format!(
                    "{}/devices/{}/subscriptions",
                    server, subscriber_uuid
                ))
                .header("Authorization", auth)
                .json(&body)
                .send()
                .await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Token { cmd } => match cmd {
            TokenCommands::Generate {
                uuid,
                expires_on,
                tag,
            } => {
                let target = uuid
                    .or_else(|| creds.uuid.clone())
                    .ok_or_else(|| anyhow::anyhow!("UUID required"))?;
                let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
                let mut opts = serde_json::json!({});
                if let Some(e) = expires_on {
                    opts["expiresOn"] = e.into();
                }
                if let Some(t) = tag {
                    opts["tag"] = t.into();
                }
                let resp = client
                    .post(format!("{}/devices/{}/tokens", server, target))
                    .header("Authorization", auth)
                    .json(&opts)
                    .send()
                    .await?;
                let result: Value = resp.json().await?;
                print_value(&result, &cli.format);
            }
            TokenCommands::Revoke { uuid, token } => {
                let target = uuid
                    .or_else(|| creds.uuid.clone())
                    .ok_or_else(|| anyhow::anyhow!("UUID required"))?;
                let auth = auth_header().ok_or_else(|| anyhow::anyhow!("No credentials"))?;
                let resp = client
                    .delete(format!("{}/devices/{}/tokens/{}", server, target, token))
                    .header("Authorization", auth)
                    .send()
                    .await?;
                let result: Value = resp.json().await?;
                print_value(&result, &cli.format);
            }
        },

        Commands::Status => {
            let resp = client.get(format!("{}/status", server)).send().await?;
            let result: Value = resp.json().await?;
            print_value(&result, &cli.format);
        }

        Commands::Config => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "server": creds.server,
                    "uuid": creds.uuid,
                    "token": creds.token.as_ref().map(|t| {
                        // Show only first 8 chars
                        format!("{}...", &t[..t.len().min(8)])
                    })
                }))?
            );
        }
    }

    Ok(())
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };
        out.push(CHARS[b0 >> 2] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((b1 & 15) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[b2 & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}
