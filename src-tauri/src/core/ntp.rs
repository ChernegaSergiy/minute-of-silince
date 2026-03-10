//! Lightweight NTP offset query.
//!
//! Returns the signed offset in milliseconds between the NTP server time and
//! the local system clock.  A positive value means the local clock is behind.

use crate::error::{AppError, Result};

/// Query an NTP server and return the clock offset in milliseconds.
pub async fn query_offset(server: &str) -> Result<i64> {
    // `sntp` is a synchronous crate; run it on the blocking thread pool.
    let server = server.to_owned();
    tokio::task::spawn_blocking(move || query_offset_sync(&server))
        .await
        .map_err(|e| AppError::Ntp(e.to_string()))?
}

fn query_offset_sync(server: &str) -> Result<i64> {
    // Build the NTP address with the default port.
    let addr = format!("{server}:123");

    match sntp::sync(addr.as_str()) {
        Ok(result) => {
            // `result.offset` is a `chrono::Duration`.
            Ok(result.offset.num_milliseconds())
        }
        Err(e) => Err(AppError::Ntp(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test requires network access; skip in CI with `--skip ntp`.
    #[tokio::test]
    #[ignore = "requires network"]
    async fn pool_ntp_org_responds() {
        let offset = query_offset("pool.ntp.org").await;
        assert!(offset.is_ok(), "NTP query failed: {:?}", offset.err());
        let ms = offset.unwrap();
        // Sanity: offset should be within ±60 seconds on any sane machine.
        assert!(ms.abs() < 60_000, "Suspiciously large NTP offset: {ms} ms");
    }
}
