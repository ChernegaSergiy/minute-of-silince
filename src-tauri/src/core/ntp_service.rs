//! NTP service with caching and periodic synchronization.
//!
//! Provides a clean OOP interface for NTP time synchronization,
//! supporting cached offsets and configurable sync intervals.

use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::{AppError, Result};

const DEFAULT_SYNC_INTERVAL: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone)]
pub struct NtpService {
    server: String,
    cached_offset: Arc<Mutex<Option<i64>>>,
    last_sync: Arc<Mutex<Option<DateTime<Local>>>>,
    sync_interval: Duration,
}

impl NtpService {
    pub fn new(server: String) -> Self {
        Self {
            server,
            cached_offset: Arc::new(Mutex::new(None)),
            last_sync: Arc::new(Mutex::new(None)),
            sync_interval: DEFAULT_SYNC_INTERVAL,
        }
    }

    pub fn sync(&self) -> impl std::future::Future<Output = Result<i64>> + Send + '_ {
        async move {
            let offset = self.sync_once().await?;
            self.set_cached(offset);
            self.set_last_sync(Local::now());
            Ok(offset)
        }
    }

    pub fn get_offset(&self) -> Option<i64> {
        *self.cached_offset.lock().unwrap()
    }

    pub fn should_sync(&self) -> bool {
        if let Some(last) = self.last_sync_time() {
            let elapsed = Local::now() - last;
            elapsed.num_seconds() as u64 >= self.sync_interval.as_secs()
        } else {
            true
        }
    }

    pub fn last_sync_time(&self) -> Option<DateTime<Local>> {
        *self.last_sync.lock().unwrap()
    }

    pub fn is_synced(&self) -> bool {
        self.cached_offset.lock().unwrap().is_some()
    }

    pub fn update_cached(&self, offset: i64) {
        self.set_cached(offset);
        self.set_last_sync(Local::now());
    }

    pub fn clear_cache(&self) {
        *self.cached_offset.lock().unwrap() = None;
        *self.last_sync.lock().unwrap() = None;
    }

    pub fn server(&self) -> String {
        self.server.clone()
    }

    async fn sync_once(&self) -> Result<i64> {
        let server = self.server.clone();
        tokio::task::spawn_blocking(move || {
            let result = rsntp::SntpClient::new()
                .synchronize(&server)
                .map_err(|e| AppError::Ntp(e.to_string()))?;
            let offset_ms = result.clock_offset().as_secs_f64() * 1000.0;
            Ok(offset_ms as i64)
        })
        .await
        .map_err(|e| AppError::Ntp(e.to_string()))?
    }

    fn set_cached(&self, offset: i64) {
        *self.cached_offset.lock().unwrap() = Some(offset);
    }

    fn set_last_sync(&self, time: DateTime<Local>) {
        *self.last_sync.lock().unwrap() = Some(time);
    }
}

impl Default for NtpService {
    fn default() -> Self {
        Self::new("pool.ntp.org".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntp_service_initialization() {
        let service = NtpService::new("pool.ntp.org".to_string());
        assert!(!service.is_synced());
        assert!(service.last_sync_time().is_none());
    }
}
