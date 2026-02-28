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
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
                println!("{BOLD}{BLUE}bridge-echo{RESET}");
                println!();
                println!("{RED}● offline{RESET} {DIM}— {e}{RESET}");
                if once {
                    std::process::exit(1);
                }
            }
        }

        if once {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn render(data: &serde_json::Value) {
    let active = data["active"].as_array();
    let active_count = active.map(|a| a.len()).unwrap_or(0);

    println!("{BOLD}{BLUE}bridge-echo{RESET}");
    println!();

    if active_count == 0 {
        println!("{GREEN}● idle{RESET}");
    } else {
        for req in active.unwrap() {
            let channel = req["channel"].as_str().unwrap_or("?");
            let preview = req["message_preview"].as_str().unwrap_or("");
            let elapsed = req["elapsed_secs"].as_u64().unwrap_or(0);

            let elapsed_str = fmt_duration(elapsed);
            let (color, label) = if elapsed >= 600 {
                (RED, "stuck")
            } else if elapsed >= 300 {
                (ORANGE, "active")
            } else {
                (GREEN, "active")
            };

            println!(
                "{color}● {label}{RESET}  {color}{elapsed_str}{RESET}  {PURPLE}{channel}{RESET}"
            );
            if !preview.is_empty() {
                println!("  {GRAY}{preview}{RESET}");
            }
        }
    }
}

fn fmt_duration(secs: u64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    if m > 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}
