use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use uuid::Uuid;

use freshblu_core::error::FreshBluError;

/// Sliding-window rate limiter per device UUID.
pub struct RateLimiter {
    windows: DashMap<Uuid, (u64, Instant)>,
    max_requests: u64,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_secs: u64) -> Arc<Self> {
        let limiter = Arc::new(Self {
            windows: DashMap::new(),
            max_requests,
            window_secs,
        });

        // Spawn background eviction task every 5 minutes
        let weak = Arc::downgrade(&limiter);
        let window = window_secs;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                match weak.upgrade() {
                    Some(limiter) => limiter.evict_stale(window),
                    None => break, // RateLimiter dropped, stop task
                }
            }
        });

        limiter
    }

    /// Check if a request is allowed. Returns Ok(()) or Err(RateLimitExceeded).
    pub fn check(&self, uuid: &Uuid) -> Result<(), FreshBluError> {
        let now = Instant::now();
        let mut entry = self.windows.entry(*uuid).or_insert((0, now));
        let (count, window_start) = entry.value_mut();

        // Reset window if expired
        if now.duration_since(*window_start).as_secs() >= self.window_secs {
            *count = 0;
            *window_start = now;
        }

        *count += 1;
        if *count > self.max_requests {
            return Err(FreshBluError::RateLimitExceeded);
        }

        Ok(())
    }

    /// Remove entries whose windows have expired (stale for > 2× window).
    fn evict_stale(&self, window_secs: u64) {
        let now = Instant::now();
        let threshold = window_secs * 2;
        self.windows
            .retain(|_, (_, start)| now.duration_since(*start).as_secs() < threshold);
    }

    /// Number of tracked devices (for testing/metrics).
    pub fn tracked_count(&self) -> usize {
        self.windows.len()
    }
}
