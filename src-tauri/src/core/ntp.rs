//! Lightweight NTP offset query.
//!
//! Returns the signed offset in milliseconds between the NTP server time and
//! the local system clock.  A positive value means the local clock is behind.

use crate::error::{AppError, Result};

/// Query an NTP server and return the clock offset in milliseconds.
pub async fn query_offset(server: &str) -> Result<i64> {
    let server = server.to_owned();
    tokio::task::spawn_blocking(move || query_offset_sync(&server))
        .await
        .map_err(|e| AppError::Ntp(e.to_string()))?
}

fn query_offset_sync(server: &str) -> Result<i64> {
    let result = rsntp::SntpClient::new()
        .synchronize(server)
        .map_err(|e| AppError::Ntp(e.to_string()))?;

    let offset_ms = result.clock_offset().as_secs_f64() * 1000.0;
    Ok(offset_ms as i64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rsntp_library_works() {
        let result = rsntp::SntpClient::new().synchronize("pool.ntp.org");
        assert!(result.is_ok(), "NTP query failed: {:?}", result.err());
    }
}
