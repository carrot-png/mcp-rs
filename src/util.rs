pub use rmcp::model::CallToolResult;
use rmcp::model::Content;

pub fn success(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(msg)])
}

pub fn error(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg)])
}
