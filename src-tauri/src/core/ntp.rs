//! NTP query tests.
//!
//! Legacy functions moved to ntp_service.rs.

#[cfg(test)]
mod tests {
    #[test]
    fn rsntp_library_works() {
        let result = rsntp::SntpClient::new().synchronize("pool.ntp.org");
        assert!(result.is_ok(), "NTP query failed: {:?}", result.err());
    }
}
