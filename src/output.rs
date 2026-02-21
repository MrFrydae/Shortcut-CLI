use std::fmt;
use std::io::Write;
use std::sync::{Arc, Mutex};

use colored::Colorize;

// ── Output mode ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum OutputMode {
    Human,
    Json,
    Quiet,
    Format(String),
}

// ── Color mode ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

// ── Shared buffer (for tests) ────────────────────────────────────────

pub type SharedBuffer = Arc<Mutex<Vec<u8>>>;

// ── OutputConfig ─────────────────────────────────────────────────────

pub struct OutputConfig {
    pub mode: OutputMode,
    pub color_mode: ColorMode,
    pub dry_run: bool,
    writer: Mutex<Box<dyn Write + Send>>,
}

impl OutputConfig {
    /// Create an OutputConfig that writes to stdout.
    pub fn new(mode: OutputMode, color_mode: ColorMode) -> Self {
        Self {
            mode,
            color_mode,
            dry_run: false,
            writer: Mutex::new(Box::new(std::io::stdout())),
        }
    }

    /// Create an OutputConfig backed by a shared buffer (for tests).
    pub fn with_buffer(mode: OutputMode, color_mode: ColorMode) -> (Self, SharedBuffer) {
        let buf: SharedBuffer = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedWriter(Arc::clone(&buf));
        let config = Self {
            mode,
            color_mode,
            dry_run: false,
            writer: Mutex::new(Box::new(writer)),
        };
        (config, buf)
    }

    /// Enable or disable dry-run mode.
    pub fn with_dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Write a formatted line to the output.
    pub fn writeln(&self, args: fmt::Arguments<'_>) -> Result<(), Box<dyn std::error::Error>> {
        let mut w = self.writer.lock().unwrap();
        writeln!(w, "{}", args)?;
        Ok(())
    }

    /// Write formatted content without a trailing newline.
    pub fn write_str(&self, args: fmt::Arguments<'_>) -> Result<(), Box<dyn std::error::Error>> {
        let mut w = self.writer.lock().unwrap();
        write!(w, "{}", args)?;
        Ok(())
    }

    pub fn is_json(&self) -> bool {
        matches!(self.mode, OutputMode::Json)
    }

    pub fn is_quiet(&self) -> bool {
        matches!(self.mode, OutputMode::Quiet)
    }

    pub fn is_format(&self) -> bool {
        matches!(self.mode, OutputMode::Format(_))
    }

    pub fn format_template(&self) -> Option<&str> {
        match &self.mode {
            OutputMode::Format(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Print a dry-run summary of the request that would be sent.
    pub fn dry_run_request<T: serde::Serialize>(
        &self,
        method: &str,
        path: &str,
        body: Option<&T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.writeln(format_args!("[dry-run] {method} {path}"))?;
        if let Some(body) = body {
            let json = serde_json::to_string_pretty(body)?;
            self.writeln(format_args!("{json}"))?;
        }
        Ok(())
    }

    pub fn use_color(&self) -> bool {
        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

// ── SharedWriter ─────────────────────────────────────────────────────

struct SharedWriter(SharedBuffer);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

// ── Table ────────────────────────────────────────────────────────────

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: Vec<&str>) -> Self {
        Self {
            headers: headers.into_iter().map(|h| h.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn render(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }

        let col_count = self.headers.len();
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_count {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        let mut out = String::new();
        // Header row
        for (i, header) in self.headers.iter().enumerate() {
            if i > 0 {
                out.push_str("  ");
            }
            out.push_str(&format!("{:<width$}", header, width = widths[i]));
        }
        out.push('\n');

        // Data rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i >= col_count {
                    break;
                }
                if i > 0 {
                    out.push_str("  ");
                }
                // Right-align the first column (ID) if all values are numeric
                if i == 0 {
                    out.push_str(&format!("{:>width$}", cell, width = widths[i]));
                } else {
                    out.push_str(&format!("{:<width$}", cell, width = widths[i]));
                }
            }
            out.push('\n');
        }

        out
    }
}

// ── Format template ──────────────────────────────────────────────────

/// Replace `{field}` placeholders in `template` with values from a serializable item.
/// Supports dot notation for nested fields: `{stats.num_stories}`.
pub fn format_template<T: serde::Serialize>(
    template: &str,
    item: &T,
) -> Result<String, Box<dyn std::error::Error>> {
    let value = serde_json::to_value(item)?;
    let mut result = template.to_string();

    // Find all {field} placeholders
    let mut start = 0;
    loop {
        let Some(open) = result[start..].find('{') else {
            break;
        };
        let open = start + open;
        let Some(close) = result[open..].find('}') else {
            break;
        };
        let close = open + close;
        let field = &result[open + 1..close];

        // Resolve the field value via dot notation
        let resolved = resolve_field(&value, field);
        let replacement = match &resolved {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };

        result.replace_range(open..=close, &replacement);
        start = open + replacement.len();
    }

    Ok(result)
}

fn resolve_field(value: &serde_json::Value, path: &str) -> serde_json::Value {
    let mut current = value;
    for part in path.split('.') {
        match current.get(part) {
            Some(v) => current = v,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

// ── Color style helpers ──────────────────────────────────────────────

pub fn style_id(id: impl fmt::Display) -> String {
    format!("{}", id.to_string().bold())
}

pub fn style_story_type(story_type: &str) -> String {
    match story_type {
        "bug" => format!("{}", story_type.red()),
        "feature" => format!("{}", story_type.green()),
        "chore" => format!("{}", story_type.yellow()),
        _ => story_type.to_string(),
    }
}

pub fn style_state_type(state_type: &str) -> String {
    match state_type {
        "unstarted" => format!("{}", state_type.dimmed()),
        "started" => format!("{}", state_type.cyan()),
        "done" => format!("{}", state_type.green()),
        _ => state_type.to_string(),
    }
}

pub fn style_mention(mention: &str) -> String {
    format!("{}", mention.bold().cyan())
}

// ── Macros ───────────────────────────────────────────────────────────

/// Convenience macro for `out.writeln(format_args!(...))`.
#[macro_export]
macro_rules! out_println {
    ($out:expr, $($arg:tt)*) => {
        $out.writeln(format_args!($($arg)*))?
    };
}

/// Convenience macro for `out.write_str(format_args!(...))`.
#[macro_export]
macro_rules! out_print {
    ($out:expr, $($arg:tt)*) => {
        $out.write_str(format_args!($($arg)*))?
    };
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_template_basic() {
        let item = serde_json::json!({"id": 42, "name": "Test Story"});
        let result = format_template("{id} - {name}", &item).unwrap();
        assert_eq!(result, "42 - Test Story");
    }

    #[test]
    fn format_template_nested() {
        let item = serde_json::json!({"id": 1, "stats": {"num_stories": 5}});
        let result = format_template("{id}: {stats.num_stories} stories", &item).unwrap();
        assert_eq!(result, "1: 5 stories");
    }

    #[test]
    fn format_template_missing_field() {
        let item = serde_json::json!({"id": 1});
        let result = format_template("{id} - {missing}", &item).unwrap();
        assert_eq!(result, "1 - ");
    }

    #[test]
    fn table_render_alignment() {
        let mut table = Table::new(vec!["ID", "Name"]);
        table.add_row(vec!["1".to_string(), "Alpha".to_string()]);
        table.add_row(vec!["100".to_string(), "Beta".to_string()]);
        let rendered = table.render();
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 3);
        // Header
        assert!(lines[0].contains("ID"));
        assert!(lines[0].contains("Name"));
        // IDs should be right-aligned
        assert!(lines[1].contains("  1"));
        assert!(lines[2].contains("100"));
    }

    #[test]
    fn table_render_empty() {
        let table = Table::new(vec!["ID", "Name"]);
        assert_eq!(table.render(), "");
    }

    #[test]
    fn color_helpers() {
        // Test with color disabled - no ANSI codes
        colored::control::set_override(false);
        assert_eq!(style_id(42), "42");
        assert_eq!(style_story_type("bug"), "bug");
        assert_eq!(style_state_type("done"), "done");
        assert_eq!(style_mention("@alice"), "@alice");

        // Test with color enabled - should contain ANSI escape codes
        colored::control::set_override(true);
        assert!(style_id(42).contains("\x1b["));
        assert!(style_story_type("bug").contains("\x1b["));

        // Reset
        colored::control::unset_override();
    }

    #[test]
    fn output_config_write_to_buffer() {
        let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
        out.writeln(format_args!("hello {}", "world")).unwrap();
        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "hello world\n");
    }

    #[test]
    fn dry_run_default_false() {
        let out = OutputConfig::new(OutputMode::Human, ColorMode::Never);
        assert!(!out.is_dry_run());
    }

    #[test]
    fn dry_run_with_builder() {
        let out = OutputConfig::new(OutputMode::Human, ColorMode::Never).with_dry_run(true);
        assert!(out.is_dry_run());
    }

    #[test]
    fn dry_run_buffer_default_false() {
        let (out, _buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
        assert!(!out.is_dry_run());
    }

    #[test]
    fn dry_run_request_with_body() {
        let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
        let out = out.with_dry_run(true);
        let body = serde_json::json!({"name": "Test", "type": "bug"});
        out.dry_run_request("POST", "/api/v3/stories", Some(&body))
            .unwrap();
        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert!(output.contains("[dry-run] POST /api/v3/stories"));
        assert!(output.contains("\"name\": \"Test\""));
        assert!(output.contains("\"type\": \"bug\""));
    }

    #[test]
    fn dry_run_request_without_body() {
        let (out, buf) = OutputConfig::with_buffer(OutputMode::Human, ColorMode::Never);
        let out = out.with_dry_run(true);
        out.dry_run_request::<serde_json::Value>("DELETE", "/api/v3/stories/42", None)
            .unwrap();
        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "[dry-run] DELETE /api/v3/stories/42\n");
    }

    #[test]
    fn output_config_mode_checks() {
        let out = OutputConfig::new(OutputMode::Json, ColorMode::Never);
        assert!(out.is_json());
        assert!(!out.is_quiet());

        let out = OutputConfig::new(OutputMode::Quiet, ColorMode::Never);
        assert!(out.is_quiet());
        assert!(!out.is_json());

        let out = OutputConfig::new(OutputMode::Format("test".into()), ColorMode::Never);
        assert!(out.is_format());
        assert_eq!(out.format_template(), Some("test"));
    }
}
