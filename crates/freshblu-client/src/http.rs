use crate::{Error, FreshBluClient, GenerateTokenResponse, StatusResponse};
use freshblu_core::device::{DeviceView, RegisterResponse};
use freshblu_core::subscription::{Subscription, SubscriptionType};
use serde_json::Value;
use uuid::Uuid;

impl FreshBluClient {
    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url)
            .header("Content-Type", "application/json");

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();

        if status == 204 {
            return Ok(Value::Null);
        }

        let text = resp.text().await?;

        if status >= 400 {
            let message = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or(text);
            return Err(Error::Http { status, message });
        }

        serde_json::from_str(&text).map_err(Error::from)
    }

    // -- Device management --

    /// Register a new device. Returns the device with a plaintext token.
    pub async fn register(&self, properties: Value) -> Result<RegisterResponse, Error> {
        let val = self.request(reqwest::Method::POST, "/devices", Some(properties)).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Get the authenticated device's info.
    pub async fn whoami(&self) -> Result<DeviceView, Error> {
        let val = self.request(reqwest::Method::GET, "/whoami", None).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Get a device by UUID.
    pub async fn get_device(&self, uuid: &Uuid) -> Result<DeviceView, Error> {
        let val = self.request(reqwest::Method::GET, &format!("/devices/{}", uuid), None).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Update a device's properties.
    pub async fn update_device(&self, uuid: &Uuid, properties: Value) -> Result<DeviceView, Error> {
        let val = self.request(reqwest::Method::PUT, &format!("/devices/{}", uuid), Some(properties)).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Unregister (delete) a device.
    pub async fn unregister(&self, uuid: &Uuid) -> Result<(), Error> {
        self.request(reqwest::Method::DELETE, &format!("/devices/{}", uuid), None).await?;
        Ok(())
    }

    /// Search for devices.
    pub async fn search(&self, query: Value) -> Result<Vec<DeviceView>, Error> {
        let val = self.request(reqwest::Method::POST, "/devices/search", Some(query)).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Get devices owned by the authenticated device.
    pub async fn my_devices(&self) -> Result<Vec<DeviceView>, Error> {
        let val = self.request(reqwest::Method::GET, "/mydevices", None).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Claim an unclaimed device.
    pub async fn claim_device(&self, uuid: &Uuid) -> Result<DeviceView, Error> {
        let val = self.request(reqwest::Method::POST, &format!("/claimdevice/{}", uuid), None).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    // -- Messaging --

    /// Send a message to specific devices.
    pub async fn message(&self, devices: &[&str], payload: Value) -> Result<(), Error> {
        let body = serde_json::json!({
            "devices": devices,
            "payload": payload,
        });
        self.request(reqwest::Method::POST, "/messages", Some(body)).await?;
        Ok(())
    }

    /// Broadcast a message to all subscribers.
    pub async fn broadcast(&self, payload: Value) -> Result<(), Error> {
        let body = serde_json::json!({ "payload": payload });
        self.request(reqwest::Method::POST, "/broadcasts", Some(body)).await?;
        Ok(())
    }

    // -- Subscriptions --

    /// Create a subscription.
    pub async fn create_subscription(
        &self,
        subscriber: &Uuid,
        emitter: &Uuid,
        sub_type: SubscriptionType,
    ) -> Result<Subscription, Error> {
        let body = serde_json::json!({
            "subscriberUuid": subscriber,
            "emitterUuid": emitter,
            "type": sub_type,
        });
        let val = self.request(
            reqwest::Method::POST,
            &format!("/devices/{}/subscriptions", subscriber),
            Some(body),
        ).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Delete a subscription.
    pub async fn delete_subscription(
        &self,
        subscriber: &Uuid,
        emitter: &Uuid,
        sub_type: SubscriptionType,
    ) -> Result<(), Error> {
        let type_str = format!("{:?}", sub_type)
            .chars()
            .map(|c| if c == '.' { '-' } else { c })
            .collect::<String>();
        self.request(
            reqwest::Method::DELETE,
            &format!("/devices/{}/subscriptions/{}/{}", subscriber, emitter, type_str),
            None,
        ).await?;
        Ok(())
    }

    /// List subscriptions for a device.
    pub async fn subscriptions(&self, subscriber: &Uuid) -> Result<Vec<Subscription>, Error> {
        let val = self.request(
            reqwest::Method::GET,
            &format!("/devices/{}/subscriptions", subscriber),
            None,
        ).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    // -- Tokens --

    /// Generate a new token for a device.
    pub async fn generate_token(&self, uuid: &Uuid) -> Result<GenerateTokenResponse, Error> {
        let val = self.request(
            reqwest::Method::POST,
            &format!("/devices/{}/tokens", uuid),
            Some(serde_json::json!({})),
        ).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    /// Revoke a specific token.
    pub async fn revoke_token(&self, uuid: &Uuid, token: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/devices/{}/tokens/{}", uuid, token),
            None,
        ).await?;
        Ok(())
    }

    /// Reset all tokens for a device, returning a new one.
    pub async fn reset_token(&self, uuid: &Uuid) -> Result<GenerateTokenResponse, Error> {
        let val = self.request(
            reqwest::Method::POST,
            &format!("/devices/{}/token", uuid),
            None,
        ).await?;
        serde_json::from_value(val).map_err(Error::from)
    }

    // -- Status --

    /// Get server health status (no auth required).
    pub async fn status(&self) -> Result<StatusResponse, Error> {
        let val = self.request(reqwest::Method::GET, "/status", None).await?;
        serde_json::from_value(val).map_err(Error::from)
    }
}
