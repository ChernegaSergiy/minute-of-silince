pub mod commands;
pub mod tray;

use chrono::{DateTime, Local, NaiveDate, NaiveTime};

pub(crate) fn next_skip_date(now: DateTime<Local>) -> NaiveDate {
    let ceremony_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    if now.time() < ceremony_time {
        now.date_naive()
    } else {
        (now + chrono::Duration::days(1)).date_naive()
    }
}
