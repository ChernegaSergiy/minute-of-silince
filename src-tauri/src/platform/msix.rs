use std::env;

pub fn is_msix_package() -> bool {
    env::var("APP_ID").is_ok()
}