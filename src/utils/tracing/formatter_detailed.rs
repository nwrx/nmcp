use std::fmt;
use tracing::{Event, Subscriber};
use tracing_core::field::{Field, Visit};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

use crate::Backtrace;

/// A custom formatter that provides detailed error information including backtraces.
/// It will show up to 3 frames from the backtrace when errors are logged.
#[derive(Debug, Clone, Copy)]
pub struct FormatterDetailed;

/// Visits fields in an event to extract error information
struct DetailedFieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl DetailedFieldVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl Visit for DetailedFieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields
            .push((field.name().to_string(), format!("{value:?}")));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}

impl<S, N> FormatEvent<S, N> for FormatterDetailed
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        match *metadata.level() {
            tracing::Level::ERROR => write!(writer, "\x1b[41;97m ✖ \x1b[0m")?,
            tracing::Level::WARN => write!(writer, "\x1b[43;97m ⚠ \x1b[0m")?,
            tracing::Level::INFO => write!(writer, "\x1b[42;97m ℹ \x1b[0m")?,
            tracing::Level::DEBUG => write!(writer, "\x1b[44;97m ⋯ \x1b[0m")?,
            tracing::Level::TRACE => write!(writer, "\x1b[45;97m » \x1b[0m")?,
        };

        // --- Push the current span context to the writer.
        let current_time = chrono::Local::now();
        let current_time = current_time.format("%Y-%m-%d %H:%M:%S%.3f");
        let current_time = format!("\x1b[100;30m {current_time} \x1b[0m");
        write!(writer, "{current_time}")?;

        // --- Push the event's target to the writer.
        if let Some(scope) = ctx.event_scope() {
            let spans: Vec<_> = scope.from_root().map(|span| span.name()).collect();
            let spans = spans.join(" ▶ ");
            write!(writer, "\x1b[1;43m {spans} \x1b[0m")?;
        };

        // --- Visit event fields to extract error information
        let mut visitor = DetailedFieldVisitor::new();
        event.record(&mut visitor);

        // --- Write the message
        match visitor.message {
            Some(message) => write!(writer, " \x1b[1m{message}\x1b[0m")?,
            None => write!(writer, " \x1b[1m{}\x1b[0m", metadata.target())?,
        }

        // --- Write fields, handling error fields specially
        if visitor.fields.is_empty() {
            return writeln!(writer);
        }

        writeln!(writer, "\n\x1b[90m│\x1b[0m")?;
        for (key, value) in &visitor.fields {
            match key.as_str() {
                "error.name" => write!(
                    writer,
                    "\n\x1b[90m│\x1b[0m  \x1b[31m  ⚠ NAME\x1b[0m:    {value}"
                )?,
                "error.code" => write!(
                    writer,
                    "\n\x1b[90m│\x1b[0m  \x1b[31m  ⦿ STATUS\x1b[0m:  {value}"
                )?,
                "error.message" => write!(
                    writer,
                    "\n\x1b[90m│\x1b[0m  \x1b[31m  ✘ MESSAGE\x1b[0m: {value}"
                )?,
                "error.backtrace" => {
                    write!(writer, "\n\n  \x1b[33m  ⧉ BACKTRACE\x1b[0m")?;
                    let backtrace: Backtrace = serde_json::from_str(value).unwrap_or_default();
                    let mut output = String::new();
                    for frame in backtrace.frames {
                        if let Some(filename) = frame.filename.as_deref() {
                            if filename.starts_with("/") || filename.starts_with("\\") {
                                continue;
                            }
                        }
                        output.push_str(&format!(
                            "\n{}:{}:{}",
                            frame
                                .filename
                                .as_deref()
                                .map(|p| p.to_string_lossy())
                                .unwrap_or_default(),
                            frame.lineno.unwrap_or_default(),
                            frame.colno.unwrap_or_default(),
                        ));
                    }
                    write_yellow_box(&mut writer, &output)?;
                }
                _ => {
                    let key = key.to_uppercase();
                    let mut lines = Vec::new();
                    for (i, line) in value.lines().enumerate() {
                        match i {
                            0 => lines.push(line.to_string()),
                            _ => lines.push(format!("\n\x1b[90m│\x1b[0m    {line}")),
                        }
                    }
                    let value = lines.join("");
                    writeln!(writer, "\x1b[90m│\x1b[0m  \x1b[90m{key}:\x1b[0m {value}")?;
                }
            }
        }
        writeln!(writer, "\x1b[90m│\x1b[0m")
    }
}

fn write_yellow_box(
    writer: &mut tracing_subscriber::fmt::format::Writer<'_>,
    text: &str,
) -> fmt::Result {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }

    // Find the maximum line length for consistent box width
    let max_line_len = lines.iter().map(|line| line.len()).max().unwrap_or(0);

    // Add some padding for the box
    let box_width = max_line_len + 4; // Adjusted width to match sides

    // Write the opening line with top corners
    write!(writer, "\n  \x1b[33m╭")?;
    for _ in 0..(box_width - 2) {
        write!(writer, "─")?;
    }
    writeln!(writer, "╮\x1b[0m")?;

    // Write each line with padding
    for line in lines {
        // Add spaces to pad the line to max_line_len
        let padding = " ".repeat(max_line_len - line.len());
        writeln!(
            writer,
            "  \x1b[33m│\x1b[0m {line}{padding} \x1b[33m│\x1b[0m"
        )?;
    }

    // Write the closing line with bottom corners
    write!(writer, "  \x1b[33m╰")?;
    for _ in 0..(box_width - 2) {
        write!(writer, "─")?;
    }
    writeln!(writer, "╯\x1b[0m")?;

    Ok(())
}
