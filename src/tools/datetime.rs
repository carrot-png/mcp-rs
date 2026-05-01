use time::{OffsetDateTime, format_description::BorrowedFormatItem, macros::format_description};

use crate::util::{CallToolResult, success};

const FORMAT: &[BorrowedFormatItem<'_>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]"
);

pub fn datetime() -> CallToolResult {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    success(now.format(FORMAT).unwrap())
}
