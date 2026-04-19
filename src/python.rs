use std::time::Duration;

use monty::{LimitedTracker, MontyException, MontyRun, PrintWriter, ResourceLimits};
use rmcp::model::CallToolResult;

use crate::util::{error, success};

const MAX_MEMORY_BYTES: usize = 1024 * 1024 * 1024; // 1 GiB
const MAX_DURATION_SECS: Duration = Duration::from_secs(10);
const MAX_OUTPUT_CHARS: usize = 1024;

pub fn run_python(code: String) -> CallToolResult {
    match run_monty(code) {
        Ok(out) => success(out),
        Err(err) => error(err.message().unwrap_or("Failed to run python script.")),
    }
}

fn get_limits() -> ResourceLimits {
    ResourceLimits::new()
        .max_memory(MAX_MEMORY_BYTES)
        .max_duration(MAX_DURATION_SECS)
}

fn run_monty(code: String) -> Result<String, MontyException> {
    println!("\nRunning Python:\n\x1b[2m{code}\x1b[0m");

    let tracker = LimitedTracker::new(get_limits());
    let runner = MontyRun::new(code, "script.py", vec![])?;
    let mut out = String::new();
    runner.run(vec![], tracker, PrintWriter::CollectString(&mut out))?;

    if out.len() > MAX_OUTPUT_CHARS {
        out.truncate(MAX_OUTPUT_CHARS);
        out.push_str("... (truncated)");
    }

    println!("Result:\n\x1b[2m{out}\x1b[0m");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_python_success() {
        let code = "print(5 + 5)".to_string();
        let result = run_python(code);
        let debug_res = format!("{:?}", result);
        assert!(debug_res.contains("10"));
    }

    #[test]
    fn test_run_python_syntax_error() {
        let code = "if True".to_string();
        let result = run_python(code);
        assert!(result.is_error == Some(true));
    }
}
