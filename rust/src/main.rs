use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const TASKS_FILE: &str = "tasks.json";

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return usage_error();
    }

    let tasks_path = env::current_dir()
        .map_err(|e| format!("No se pudo obtener el directorio actual: {}", e))?
        .join(TASKS_FILE);

    let mut tasks = load_tasks(&tasks_path)?;

    match args[0].as_str() {
        "add" => {
            if args.len() != 2 {
                return Err("Uso: task-tracker add \"descripción\"".into());
            }
            let desc = args[1].trim();
            if desc.is_empty() {
                return Err("La descripción no puede estar vacía.".into());
            }
            let now = iso8601_utc_now();
            let id = next_id(&tasks);
            tasks.push(Task {
                id,
                description: desc.to_string(),
                status: Status::Todo,
                created_at: now.clone(),
                updated_at: now,
            });
            save_tasks(&tasks_path, &tasks)?;
            println!("Tarea {} creada.", id);
        }
        "update" => {
            if args.len() != 3 {
                return Err("Uso: task-tracker update <id> \"nueva descripción\"".into());
            }
            let id = parse_positive_id(&args[1])?;
            let desc = args[2].trim();
            if desc.is_empty() {
                return Err("La descripción no puede estar vacía.".into());
            }
            let now = iso8601_utc_now();
            let t = tasks
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or_else(|| format!("No existe la tarea con id {}.", id))?;
            t.description = desc.to_string();
            t.updated_at = now;
            save_tasks(&tasks_path, &tasks)?;
        }
        "delete" => {
            if args.len() != 2 {
                return Err("Uso: task-tracker delete <id>".into());
            }
            let id = parse_positive_id(&args[1])?;
            let pos = tasks
                .iter()
                .position(|t| t.id == id)
                .ok_or_else(|| format!("No existe la tarea con id {}.", id))?;
            tasks.remove(pos);
            save_tasks(&tasks_path, &tasks)?;
        }
        "mark-in-progress" => {
            if args.len() != 2 {
                return Err("Uso: task-tracker mark-in-progress <id>".into());
            }
            let id = parse_positive_id(&args[1])?;
            let now = iso8601_utc_now();
            let t = tasks
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or_else(|| format!("No existe la tarea con id {}.", id))?;
            t.status = Status::InProgress;
            t.updated_at = now;
            save_tasks(&tasks_path, &tasks)?;
        }
        "mark-done" => {
            if args.len() != 2 {
                return Err("Uso: task-tracker mark-done <id>".into());
            }
            let id = parse_positive_id(&args[1])?;
            let now = iso8601_utc_now();
            let t = tasks
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or_else(|| format!("No existe la tarea con id {}.", id))?;
            t.status = Status::Done;
            t.updated_at = now;
            save_tasks(&tasks_path, &tasks)?;
        }
        "list" => {
            let filter = match args.len() {
                1 => ListFilter::All,
                2 => ListFilter::parse(&args[1])?,
                _ => return Err("Uso: task-tracker list [todo|in-progress|done|not-done]".into()),
            };
            for t in tasks.iter().filter(|t| filter.matches(t)) {
                println!("{} | {} | {}", t.id, t.status, t.description);
            }
        }
        _ => {
            return usage_error();
        }
    }

    Ok(())
}

fn usage_error() -> Result<(), String> {
    Err(
        "Comandos: add | update | delete | mark-in-progress | mark-done | list [filtro]\n\
         Filtros de list: todo | in-progress | done | not-done"
            .into(),
    )
}

fn parse_positive_id(s: &str) -> Result<u64, String> {
    let id: u64 = s.parse().map_err(|_| {
        format!(
            "El id debe ser un entero positivo; se recibió {:?} que no es válido.",
            s
        )
    })?;
    if id == 0 {
        return Err("El id debe ser un entero positivo mayor que 0.".into());
    }
    Ok(id)
}

fn next_id(tasks: &[Task]) -> u64 {
    tasks.iter().map(|t| t.id).max().unwrap_or(0).saturating_add(1)
}

fn load_tasks(path: &PathBuf) -> Result<Vec<Task>, String> {
    if !path.exists() {
        let empty = "[]";
        fs::write(path, empty).map_err(|e| {
            format!(
                "No se pudo crear {}: {}",
                path.display(),
                e
            )
        })?;
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path)
        .map_err(|e| format!("No se pudo leer {}: {}", path.display(), e))?;
    let tasks = parse_tasks_json(&raw)?;
    validate_tasks_file(&tasks)?;
    Ok(tasks)
}

fn validate_tasks_file(tasks: &[Task]) -> Result<(), String> {
    for t in tasks {
        if t.description.trim().is_empty() {
            return Err(format!(
                "JSON inválido: la tarea {} tiene \"description\" vacía.",
                t.id
            ));
        }
    }
    for i in 0..tasks.len() {
        for j in (i + 1)..tasks.len() {
            if tasks[i].id == tasks[j].id {
                return Err(format!(
                    "JSON inválido: el id {} está repetido en el archivo.",
                    tasks[i].id
                ));
            }
        }
    }
    Ok(())
}

fn save_tasks(path: &PathBuf, tasks: &[Task]) -> Result<(), String> {
    let json = serialize_tasks(tasks)?;
    fs::write(path, json).map_err(|e| format!("No se pudo escribir {}: {}", path.display(), e))?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Todo,
    InProgress,
    Done,
}

impl Status {
    fn from_json(s: &str) -> Result<Self, String> {
        match s {
            "todo" => Ok(Status::Todo),
            "in-progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            _ => Err(format!("Estado desconocido en JSON: {:?}", s)),
        }
    }

    fn as_json_str(&self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::InProgress => "in-progress",
            Status::Done => "done",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_json_str())
    }
}

#[derive(Debug, Clone)]
struct Task {
    id: u64,
    description: String,
    status: Status,
    created_at: String,
    updated_at: String,
}

enum ListFilter {
    All,
    Todo,
    InProgress,
    Done,
    NotDone,
}

impl ListFilter {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "todo" => Ok(ListFilter::Todo),
            "in-progress" => Ok(ListFilter::InProgress),
            "done" => Ok(ListFilter::Done),
            "not-done" => Ok(ListFilter::NotDone),
            other => Err(format!(
                "Filtro de list no reconocido: {:?}. Use: todo | in-progress | done | not-done",
                other
            )),
        }
    }

    fn matches(&self, t: &Task) -> bool {
        match self {
            ListFilter::All => true,
            ListFilter::Todo => t.status == Status::Todo,
            ListFilter::InProgress => t.status == Status::InProgress,
            ListFilter::Done => t.status == Status::Done,
            ListFilter::NotDone => t.status != Status::Done,
        }
    }
}

// --- ISO 8601 UTC (solo std::time) ---

fn iso8601_utc_now() -> String {
    let now = SystemTime::now();
    let dur = now
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::new(0, 0));
    unix_seconds_to_iso8601(dur.as_secs())
}

fn unix_seconds_to_iso8601(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let tod = secs % 86400;
    let h = (tod / 3600) as u32;
    let m = ((tod % 3600) / 60) as u32;
    let s = (tod % 60) as u32;

    let (y, mo, d) = days_since_epoch_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, mo, d, h, m, s
    )
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_year(year: i32) -> i64 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

fn days_since_epoch_to_ymd(mut days: i64) -> (i32, u32, u32) {
    let mut year: i32 = 1970;
    while days >= days_in_year(year) {
        days -= days_in_year(year);
        year += 1;
    }

    let dim: [u32; 12] = [
        31,
        if is_leap_year(year) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1u32;
    for d in dim {
        let dl = d as i64;
        if days < dl {
            return (year, month, (days + 1) as u32);
        }
        days -= dl;
        month += 1;
    }
    (year, 12, 31)
}

// --- JSON manual: solo el subconjunto que generamos y aceptamos ---

fn serialize_tasks(tasks: &[Task]) -> Result<String, String> {
    let mut out = String::from("[");
    for (i, t) in tasks.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("\n  {");
        out.push_str(&format!(
            "\n    \"id\":{},\n    \"description\":{},\n    \"status\":\"{}\",\n    \"createdAt\":{},\n    \"updatedAt\":{}",
            t.id,
            json_escape_string(&t.description),
            t.status.as_json_str(),
            json_escape_string(&t.created_at),
            json_escape_string(&t.updated_at),
        ));
        out.push_str("\n  }");
    }
    out.push_str("\n]");
    Ok(out)
}

fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(&mut out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_tasks_json(input: &str) -> Result<Vec<Task>, String> {
    let mut p = Parser::new(input);
    p.skip_ws();
    if p.peek() != Some('[') {
        return Err("JSON inválido: se esperaba '[' al inicio del arreglo.".into());
    }
    p.advance();
    p.skip_ws();

    let mut tasks = Vec::new();
    if p.peek() == Some(']') {
        p.advance();
        p.skip_ws();
        if !p.eof() {
            return Err("JSON inválido: basura después del arreglo.".into());
        }
        return Ok(tasks);
    }

    loop {
        tasks.push(p.parse_task_object()?);
        p.skip_ws();
        match p.peek() {
            Some(',') => {
                p.advance();
                p.skip_ws();
            }
            Some(']') => {
                p.advance();
                break;
            }
            Some(c) => {
                return Err(format!(
                    "JSON inválido: se esperaba ',' o ']', se encontró {:?}",
                    c
                ));
            }
            None => return Err("JSON inválido: fin de archivo dentro del arreglo.".into()),
        }
    }

    p.skip_ws();
    if !p.eof() {
        return Err("JSON inválido: basura después del arreglo.".into());
    }

    Ok(tasks)
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser { s, i: 0 }
    }

    fn eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn peek(&self) -> Option<char> {
        self.s[self.i..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.i += c.len_utf8();
        }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        self.skip_ws();
        match self.peek() {
            Some(c) if c == expected => {
                self.advance();
                Ok(())
            }
            Some(c) => Err(format!(
                "JSON inválido: se esperaba {:?}, se encontró {:?}",
                expected, c
            )),
            None => Err(format!(
                "JSON inválido: fin de archivo; se esperaba {:?}",
                expected
            )),
        }
    }

    fn parse_task_object(&mut self) -> Result<Task, String> {
        self.expect_char('{')?;
        let mut id: Option<u64> = None;
        let mut description: Option<String> = None;
        let mut status: Option<Status> = None;
        let mut created_at: Option<String> = None;
        let mut updated_at: Option<String> = None;

        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.advance();
                break;
            }

            let key = self.parse_json_string()?;
            self.expect_char(':')?;
            self.skip_ws();

            match key.as_str() {
                "id" => {
                    if id.is_some() {
                        return Err("JSON inválido: id duplicado en un objeto.".into());
                    }
                    id = Some(self.parse_u64()?);
                }
                "description" => {
                    if description.is_some() {
                        return Err("JSON inválido: description duplicada.".into());
                    }
                    description = Some(self.parse_json_string()?);
                }
                "status" => {
                    if status.is_some() {
                        return Err("JSON inválido: status duplicado.".into());
                    }
                    status = Some(Status::from_json(&self.parse_json_string()?)?);
                }
                "createdAt" => {
                    if created_at.is_some() {
                        return Err("JSON inválido: createdAt duplicado.".into());
                    }
                    created_at = Some(self.parse_json_string()?);
                }
                "updatedAt" => {
                    if updated_at.is_some() {
                        return Err("JSON inválido: updatedAt duplicado.".into());
                    }
                    updated_at = Some(self.parse_json_string()?);
                }
                _ => self.skip_value()?,
            }

            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.advance();
                }
                Some('}') => continue,
                Some(c) => {
                    return Err(format!(
                        "JSON inválido: se esperaba ',' o '}}' en objeto, se encontró {:?}",
                        c
                    ));
                }
                None => return Err("JSON inválido: objeto sin cerrar.".into()),
            }
        }

        let id = id.ok_or_else(|| "JSON inválido: falta \"id\" en una tarea.".to_string())?;
        if id == 0 {
            return Err("JSON inválido: el id debe ser un entero positivo.".into());
        }
        let description = description
            .ok_or_else(|| "JSON inválido: falta \"description\" en una tarea.".to_string())?;
        let status = status.ok_or_else(|| "JSON inválido: falta \"status\" en una tarea.".to_string())?;
        let created_at = created_at
            .ok_or_else(|| "JSON inválido: falta \"createdAt\" en una tarea.".to_string())?;
        let updated_at = updated_at
            .ok_or_else(|| "JSON inválido: falta \"updatedAt\" en una tarea.".to_string())?;

        Ok(Task {
            id,
            description,
            status,
            created_at,
            updated_at,
        })
    }

    fn skip_value(&mut self) -> Result<(), String> {
        self.skip_ws();
        match self.peek() {
            Some('"') => {
                let _ = self.parse_json_string()?;
            }
            Some('{') => {
                self.advance();
                loop {
                    self.skip_ws();
                    if self.peek() == Some('}') {
                        self.advance();
                        break;
                    }
                    let _ = self.parse_json_string()?;
                    self.expect_char(':')?;
                    self.skip_value()?;
                    self.skip_ws();
                    match self.peek() {
                        Some(',') => self.advance(),
                        Some('}') => continue,
                        Some(c) => {
                            return Err(format!(
                                "JSON inválido al omitir valor anidado: {:?}",
                                c
                            ));
                        }
                        None => return Err("JSON inválido: valor anidado sin cerrar.".into()),
                    }
                }
            }
            Some('[') => {
                self.advance();
                loop {
                    self.skip_ws();
                    if self.peek() == Some(']') {
                        self.advance();
                        break;
                    }
                    self.skip_value()?;
                    self.skip_ws();
                    match self.peek() {
                        Some(',') => self.advance(),
                        Some(']') => continue,
                        Some(c) => {
                            return Err(format!(
                                "JSON inválido al omitir arreglo: {:?}",
                                c
                            ));
                        }
                        None => return Err("JSON inválido: arreglo sin cerrar.".into()),
                    }
                }
            }
            Some('t') => {
                if self.s[self.i..].starts_with("true") {
                    self.i += 4;
                } else {
                    return Err("JSON inválido: literal inesperado.".into());
                }
            }
            Some('f') => {
                if self.s[self.i..].starts_with("false") {
                    self.i += 5;
                } else {
                    return Err("JSON inválido: literal inesperado.".into());
                }
            }
            Some('n') => {
                if self.s[self.i..].starts_with("null") {
                    self.i += 4;
                } else {
                    return Err("JSON inválido: literal inesperado.".into());
                }
            }
            Some('-') | Some('0'..='9') => {
                let _ = self.parse_number_token()?;
            }
            _ => return Err("JSON inválido: no se pudo omitir un valor.".into()),
        }
        Ok(())
    }

    fn parse_u64(&mut self) -> Result<u64, String> {
        self.skip_ws();
        let start = self.i;
        if self.peek() == Some('-') {
            return Err("JSON inválido: el id no puede ser negativo.".into());
        }
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        if self.i == start {
            return Err("JSON inválido: se esperaba un número.".into());
        }
        let slice = &self.s[start..self.i];
        slice
            .parse::<u64>()
            .map_err(|_| format!("JSON inválido: número demasiado grande: {:?}", slice))
    }

    fn parse_number_token(&mut self) -> Result<(), String> {
        if self.peek() == Some('-') {
            self.advance();
        }
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.advance();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.advance();
            }
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
        }
        Ok(())
    }

    fn parse_json_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        self.expect_char('"')?;
        let mut out = String::new();
        loop {
            let c = self
                .peek()
                .ok_or_else(|| "JSON inválido: cadena sin cerrar.".to_string())?;
            if c == '"' {
                self.advance();
                break;
            }
            if c == '\\' {
                self.advance();
                let esc = self
                    .peek()
                    .ok_or_else(|| "JSON inválido: escape incompleto.".to_string())?;
                match esc {
                    '"' => {
                        out.push('"');
                        self.advance();
                    }
                    '\\' => {
                        out.push('\\');
                        self.advance();
                    }
                    '/' => {
                        out.push('/');
                        self.advance();
                    }
                    'b' => {
                        out.push('\u{0008}');
                        self.advance();
                    }
                    'f' => {
                        out.push('\u{000C}');
                        self.advance();
                    }
                    'n' => {
                        out.push('\n');
                        self.advance();
                    }
                    'r' => {
                        out.push('\r');
                        self.advance();
                    }
                    't' => {
                        out.push('\t');
                        self.advance();
                    }
                    'u' => {
                        self.advance();
                        let mut acc: u32 = 0;
                        for _ in 0..4 {
                            let h = self.peek().ok_or_else(|| {
                                "JSON inválido: \\u con menos de 4 hex dígitos.".to_string()
                            })?;
                            self.advance();
                            let v = h.to_digit(16).ok_or_else(|| {
                                format!("JSON inválido: \\u con dígito hex inválido {:?}", h)
                            })?;
                            acc = (acc << 4) | v;
                        }
                        let ch = char::from_u32(acc).ok_or_else(|| {
                            format!("JSON inválido: \\u{:04x} no es un escalar Unicode.", acc)
                        })?;
                        out.push(ch);
                    }
                    _ => {
                        return Err(format!("JSON inválido: secuencia de escape \\{:?}", esc));
                    }
                }
                continue;
            }

            self.advance(); // consume c
            // Rechazar controles sin escapar (salvo los que JSON permite en cadena UTF-8) — JSON exige escape para < U+0020
            if (c as u32) < 0x20 {
                return Err("JSON inválido: carácter de control sin escapar en cadena.".into());
            }
            out.push(c);
        }
        Ok(out)
    }
}
