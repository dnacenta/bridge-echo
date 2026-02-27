use std::env;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BLUE: &str = "\x1b[38;5;75m";
const GREEN: &str = "\x1b[38;5;78m";
const ORANGE: &str = "\x1b[38;5;208m";
const RED: &str = "\x1b[38;5;203m";
const PURPLE: &str = "\x1b[38;5;141m";
const GRAY: &str = "\x1b[38;5;243m";
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";

pub async fn run(once: bool) {
    let port = env::var("BRIDGE_ECHO_PORT").unwrap_or_else(|_| "3100".into());
    let url = format!("http://127.0.0.1:{port}/api/status");

    let client = reqwest::Client::new();

    loop {
        if !once {
            print!("{CLEAR_SCREEN}");
        }

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let data: serde_json::Value = match resp.json().await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("failed to parse response: {e}");
                        if once {
                            std::process::exit(1);
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                };
                render(&data);
            }
            Ok(resp) => {
                eprintln!("server returned {}", resp.status());
                if once {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{RED}connection failed:{RESET} {e}");
                eprintln!("{DIM}is bridge-echo running?{RESET}");
                if once {
                    std::process::exit(1);
                }
            }
        }

        if once {
            break;
        }

        println!();
        println!("{DIM}refreshing every 2s — Ctrl+C to exit{RESET}");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

fn render(data: &serde_json::Value) {
    let active = data["active"].as_array();
    let completed = data["completed"].as_array();

    println!("{BOLD}{BLUE}bridge-echo monitor{RESET}");
    println!("{DIM}─────────────────────────────────────────────────{RESET}");
    println!();

    // Active requests
    let active_count = active.map(|a| a.len()).unwrap_or(0);
    if active_count > 0 {
        println!(
            "{BOLD}{ORANGE}● {active_count} active request{}{RESET}",
            if active_count == 1 { "" } else { "s" }
        );
        println!();

        for req in active.unwrap() {
            let id = req["id"].as_u64().unwrap_or(0);
            let channel = req["channel"].as_str().unwrap_or("?");
            let preview = req["message_preview"].as_str().unwrap_or("");
            let elapsed = req["elapsed_secs"].as_u64().unwrap_or(0);

            let elapsed_str = fmt_duration(elapsed);
            let color = if elapsed >= 600 { RED } else { ORANGE };

            println!("  {BOLD}#{id}{RESET}  {PURPLE}{channel}{RESET}  {color}{elapsed_str}{RESET}");
            println!("  {GRAY}{preview}{RESET}");
            println!();
        }
    } else {
        println!("{DIM}no active requests{RESET}");
        println!();
    }

    // Completed requests
    let completed_count = completed.map(|c| c.len()).unwrap_or(0);
    println!("{BOLD}{GREEN}✓ {completed_count} completed{RESET} {DIM}(last 50){RESET}");
    println!();

    if let Some(items) = completed {
        for req in items.iter().rev().take(10) {
            let id = req["id"].as_u64().unwrap_or(0);
            let channel = req["channel"].as_str().unwrap_or("?");
            let msg = req["message_preview"].as_str().unwrap_or("");
            let resp = req["response_preview"].as_str().unwrap_or("");
            let duration = req["duration_secs"].as_u64().unwrap_or(0);

            let duration_str = fmt_duration(duration);

            println!("  {DIM}#{id}{RESET}  {PURPLE}{channel}{RESET}  {GREEN}{duration_str}{RESET}");
            println!("  {GRAY}→ {msg}{RESET}");
            println!("  {GRAY}← {resp}{RESET}");
            println!();
        }

        if completed_count > 10 {
            println!("  {DIM}... and {} more{RESET}", completed_count - 10);
        }
    }
}

fn fmt_duration(secs: u64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}
