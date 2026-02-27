use tokio::process::Command;
use tracing::warn;

pub struct ClaudeResponse {
    pub text: String,
    pub session_id: Option<String>,
}

pub async fn invoke(
    claude_bin: &str,
    prompt: &str,
    home: &str,
    session_id: Option<&str>,
    self_doc: Option<&str>,
) -> ClaudeResponse {
    let mut cmd = Command::new(claude_bin);
    cmd.arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("json")
        .arg("--dangerously-skip-permissions");

    if let Some(sid) = session_id {
        cmd.arg("-r").arg(sid);
    }

    if let Some(doc) = self_doc {
        cmd.arg("--append-system-prompt").arg(doc);
    }

    cmd.env("CLAUDE_CODE_ENTRYPOINT", "cli");
    cmd.env("HOME", home);
    cmd.current_dir(home);

    match cmd.output().await {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return ClaudeResponse {
                    text: if stderr.is_empty() {
                        "Claude returned an error.".into()
                    } else {
                        stderr
                    },
                    session_id: None,
                };
            }

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            parse_output(&stdout)
        }
        Err(e) => ClaudeResponse {
            text: format!("Error running Claude: {e}"),
            session_id: None,
        },
    }
}

fn parse_output(stdout: &str) -> ClaudeResponse {
    match serde_json::from_str::<serde_json::Value>(stdout) {
        Ok(parsed) => {
            let text = parsed
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let session_id = parsed
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(String::from);

            ClaudeResponse {
                text: if text.is_empty() {
                    "No response from Claude.".into()
                } else {
                    text
                },
                session_id,
            }
        }
        Err(e) => {
            warn!("failed to parse Claude JSON output: {e}");
            let text = stdout.trim().to_string();
            ClaudeResponse {
                text: if text.is_empty() {
                    "No response from Claude.".into()
                } else {
                    text
                },
                session_id: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json() {
        let input = r#"{"result": "Hello!", "session_id": "abc-123"}"#;
        let resp = parse_output(input);
        assert_eq!(resp.text, "Hello!");
        assert_eq!(resp.session_id, Some("abc-123".into()));
    }

    #[test]
    fn parse_json_no_session() {
        let input = r#"{"result": "Hello!"}"#;
        let resp = parse_output(input);
        assert_eq!(resp.text, "Hello!");
        assert!(resp.session_id.is_none());
    }

    #[test]
    fn parse_empty_result() {
        let input = r#"{"result": "", "session_id": "abc"}"#;
        let resp = parse_output(input);
        assert_eq!(resp.text, "No response from Claude.");
    }

    #[test]
    fn parse_invalid_json_falls_back() {
        let input = "raw text output";
        let resp = parse_output(input);
        assert_eq!(resp.text, "raw text output");
        assert!(resp.session_id.is_none());
    }

    #[test]
    fn parse_empty_falls_back() {
        let input = "";
        let resp = parse_output(input);
        assert_eq!(resp.text, "No response from Claude.");
    }
}
