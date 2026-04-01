//! NTP query tests.
//!
//! Legacy functions moved to ntp_service.rs.

#[cfg(test)]
mod tests {
    #[test]
    fn rsntp_library_works() {
        // Use a more stable NTP server for CI tests
        let result = rsntp::SntpClient::new().synchronize("time.google.com");

        // If it still fails, it might be a network issue in the CI environment.
        // We log the error but don't want to break the entire build if one NTP server is temporarily down.
        if let Err(e) = result {
            println!("NTP query failed, trying alternative: {:?}", e);
            let alt_result = rsntp::SntpClient::new().synchronize("time.cloudflare.com");
            assert!(
                alt_result.is_ok(),
                "Both NTP servers (Google and Cloudflare) failed: {:?}",
                alt_result.err()
            );
        } else {
            assert!(result.is_ok());
        }
    }
}
