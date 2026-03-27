use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
struct Task {
    id: i32,
    description: String,
    status: String,
    created_at: String,
    updated_at: String,
}

fn exit_err(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(1);
}

fn data_file_path() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| exit_err("Failed to determine current directory."));
    cwd.join("tasks.json")
}

fn now_iso_like() -> String {
    let secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(_) => 0,
    };
    unix_to_rfc3339(secs)
}

fn unix_to_rfc3339(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = sod / 3600;
    let minute = (sod % 3600) / 60;
    let second = sod % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn parse_tasks(raw: &str) -> Vec<Task> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return vec![];
    }
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        exit_err("Invalid JSON format in tasks.json.");
    }
    let mut out = Vec::new();
    let objects = split_top_level_objects(trimmed);
    for body in objects {
        let id = extract_num(body, "\"id\":").unwrap_or(0);
        let description = extract_str(body, "\"description\":").unwrap_or_default();
        let status = extract_str(body, "\"status\":").unwrap_or_else(|| "todo".to_string());
        let created_at = extract_str(body, "\"createdAt\":").unwrap_or_default();
        let updated_at = extract_str(body, "\"updatedAt\":").unwrap_or_default();
        if id > 0 {
            out.push(Task {
                id,
                description,
                status,
                created_at,
                updated_at,
            });
        }
    }
    out
}

fn split_top_level_objects(raw: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    let mut in_str = false;
    let mut escape = false;
    let mut depth = 0i32;
    let mut start = None;

    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
        } else {
            match c {
                '"' => in_str = true,
                '{' => {
                    if depth == 0 {
                        start = Some(i + 1);
                    }
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(s) = start {
                            out.push(&raw[s..i]);
                        }
                        start = None;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    if depth != 0 || in_str {
        exit_err("Invalid JSON format in tasks.json.");
    }
    out
}

fn extract_num(body: &str, key: &str) -> Option<i32> {
    let idx = body.find(key)?;
    let mut s = body[idx + key.len()..].trim_start();
    if s.starts_with(' ') {
        s = s.trim_start();
    }
    let mut num = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() || (ch == '-' && num.is_empty()) {
            num.push(ch);
        } else {
            break;
        }
    }
    num.parse::<i32>().ok()
}

fn extract_str(body: &str, key: &str) -> Option<String> {
    let idx = body.find(key)?;
    let mut s = body[idx + key.len()..].trim_start();
    if !s.starts_with('"') {
        return None;
    }
    s = &s[1..];
    let mut out = String::new();
    let mut escape = false;
    for ch in s.chars() {
        if escape {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            }
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            return Some(out);
        }
        out.push(ch);
    }
    None
}

fn read_tasks() -> Vec<Task> {
    let file = data_file_path();
    if !file.exists() {
        if fs::write(&file, "[]").is_err() {
            exit_err("Failed to create tasks.json.");
        }
        return vec![];
    }
    let raw = fs::read_to_string(&file).unwrap_or_else(|_| exit_err("Failed to read tasks.json."));
    parse_tasks(&raw)
}

fn write_tasks(tasks: &[Task]) {
    let mut out = String::from("[\n");
    for (i, t) in tasks.iter().enumerate() {
        out.push_str("  {\n");
        out.push_str(&format!("    \"id\": {},\n", t.id));
        out.push_str(&format!("    \"description\": \"{}\",\n", json_escape(&t.description)));
        out.push_str(&format!("    \"status\": \"{}\",\n", json_escape(&t.status)));
        out.push_str(&format!("    \"createdAt\": \"{}\",\n", json_escape(&t.created_at)));
        out.push_str(&format!("    \"updatedAt\": \"{}\"\n", json_escape(&t.updated_at)));
        out.push_str("  }");
        if i + 1 != tasks.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push(']');
    let file = data_file_path();
    if fs::write(file, out).is_err() {
        exit_err("Failed to write tasks.json.");
    }
}

fn parse_id(raw: &str) -> i32 {
    match raw.parse::<i32>() {
        Ok(v) if v > 0 => v,
        _ => exit_err("Invalid task ID. It must be a positive integer."),
    }
}

fn next_id(tasks: &[Task]) -> i32 {
    tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

fn add_task(description: &str) {
    let desc = description.trim();
    if desc.is_empty() {
        exit_err("Task description cannot be empty.");
    }
    let mut tasks = read_tasks();
    let now = now_iso_like();
    let task = Task {
        id: next_id(&tasks),
        description: desc.to_string(),
        status: "todo".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    tasks.push(task.clone());
    write_tasks(&tasks);
    println!("Task added successfully (ID: {})", task.id);
}

fn update_task(id: i32, description: &str) {
    let desc = description.trim();
    if desc.is_empty() {
        exit_err("Updated description cannot be empty.");
    }
    let mut tasks = read_tasks();
    for t in &mut tasks {
        if t.id == id {
            t.description = desc.to_string();
            t.updated_at = now_iso_like();
            write_tasks(&tasks);
            println!("Task {} updated successfully.", id);
            return;
        }
    }
    exit_err(&format!("Task with ID {} was not found.", id));
}

fn delete_task(id: i32) {
    let tasks = read_tasks();
    let before = tasks.len();
    let filtered: Vec<Task> = tasks.into_iter().filter(|t| t.id != id).collect();
    if filtered.len() == before {
        exit_err(&format!("Task with ID {} was not found.", id));
    }
    write_tasks(&filtered);
    println!("Task {} deleted successfully.", id);
}

fn set_status(id: i32, status: &str) {
    let mut tasks = read_tasks();
    for t in &mut tasks {
        if t.id == id {
            t.status = status.to_string();
            t.updated_at = now_iso_like();
            write_tasks(&tasks);
            println!("Task {} marked as {}.", id, status);
            return;
        }
    }
    exit_err(&format!("Task with ID {} was not found.", id));
}

fn print_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }
    for t in tasks {
        println!(
            "[{}] {} | status: {} | createdAt: {} | updatedAt: {}",
            t.id, t.description, t.status, t.created_at, t.updated_at
        );
    }
}

fn list_tasks(filter: Option<&str>) {
    let tasks = read_tasks();
    if let Some(s) = filter {
        if s == "not-done" {
            let filtered: Vec<Task> = tasks.into_iter().filter(|t| t.status != "done").collect();
            print_tasks(&filtered);
            return;
        }
        if s != "todo" && s != "in-progress" && s != "done" {
            exit_err("Invalid status filter. Use: todo, in-progress, done, not-done");
        }
        let filtered: Vec<Task> = tasks.into_iter().filter(|t| t.status == s).collect();
        print_tasks(&filtered);
        return;
    }
    print_tasks(&tasks);
}

fn usage() {
    println!(
        "Task Tracker CLI (Rust)\n\nUsage:\n  cargo run -- add \"Task description\"\n  cargo run -- update <id> \"New description\"\n  cargo run -- delete <id>\n  cargo run -- mark-in-progress <id>\n  cargo run -- mark-done <id>\n  cargo run -- list\n  cargo run -- list <todo|in-progress|done|not-done>"
    );
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" || args[0] == "help" {
        usage();
        return;
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                exit_err("Usage: cargo run -- add \"Task description\"");
            }
            add_task(&args[1]);
        }
        "update" => {
            if args.len() < 3 {
                exit_err("Usage: cargo run -- update <id> \"New description\"");
            }
            update_task(parse_id(&args[1]), &args[2]);
        }
        "delete" => {
            if args.len() < 2 {
                exit_err("Usage: cargo run -- delete <id>");
            }
            delete_task(parse_id(&args[1]));
        }
        "mark-in-progress" => {
            if args.len() < 2 {
                exit_err("Usage: cargo run -- mark-in-progress <id>");
            }
            set_status(parse_id(&args[1]), "in-progress");
        }
        "mark-done" => {
            if args.len() < 2 {
                exit_err("Usage: cargo run -- mark-done <id>");
            }
            set_status(parse_id(&args[1]), "done");
        }
        "list" => {
            if args.len() > 2 {
                exit_err("Usage: cargo run -- list [todo|in-progress|done|not-done]");
            }
            if args.len() == 2 {
                list_tasks(Some(&args[1]));
            } else {
                list_tasks(None);
            }
        }
        _ => exit_err("Unknown command. Run with --help for usage."),
    }
}
