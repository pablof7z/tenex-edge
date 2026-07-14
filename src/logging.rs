//! Daemon logging: colorised stdout + plain-text daemon.log, both live.
//!
//! When stdout is a terminal (`mosaico daemon` run directly) two layers are
//! installed: ANSI-coloured stdout and a plain-text file appended to daemon.log.
//! When stdout is not a terminal (`daemon` spawned detached with stdout
//! redirected to daemon.log) a single plain-text stdout layer suffices.
//!
//! Filter default: `mosaico=info`. Override with `RUST_LOG`.
//! Examples:
//!   RUST_LOG=mosaico=debug   (include relay/nip29 trace)
//!   RUST_LOG=mosaico=trace   (everything)

use anyhow::Result;
use std::fmt;
use std::path::Path;
use tracing::field::{Field, Visit};
use tracing::Level;
use tracing_subscriber::{
    fmt::{format::Writer, layer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

// ── field visitor ─────────────────────────────────────────────────────────────

/// Collects event fields into (message, key-value pairs) for custom rendering.
struct Fields {
    message: Option<String>,
    pairs: Vec<(String, String)>,
}

impl Fields {
    fn new() -> Self {
        Self {
            message: None,
            pairs: Vec::new(),
        }
    }
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let s = format!("{value:?}");
        if field.name() == "message" {
            self.message = Some(s);
        } else {
            self.pairs.push((field.name().to_string(), s));
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.pairs
                .push((field.name().to_string(), value.to_string()));
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.pairs
            .push((field.name().to_string(), value.to_string()));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.pairs
            .push((field.name().to_string(), value.to_string()));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.pairs
            .push((field.name().to_string(), value.to_string()));
    }
    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.pairs
            .push((field.name().to_string(), value.to_string()));
    }
}

// ── event formatter ───────────────────────────────────────────────────────────

struct DaemonFormatter;

impl<S, N> FormatEvent<S, N> for DaemonFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        use owo_colors::OwoColorize as _;

        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let ss = secs % 60;
        let mm = (secs / 60) % 60;
        let hh = (secs / 3600) % 24;

        let mut fields = Fields::new();
        event.record(&mut fields);

        let ansi = writer.has_ansi_escapes();
        let level = *event.metadata().level();

        if ansi {
            // dim gray timestamp
            write!(writer, "{}", format!("{hh:02}:{mm:02}:{ss:02}").dimmed())?;
            write!(writer, " ")?;

            // colored level badge with fixed width
            match level {
                Level::ERROR => write!(writer, "{}", " ERR ".on_red().white().bold())?,
                Level::WARN => write!(writer, "{}", " WRN ".on_yellow().black().bold())?,
                Level::INFO => write!(writer, "{}", " INF ".on_cyan().black().bold())?,
                Level::DEBUG => write!(writer, "{}", " DBG ".on_bright_black().white())?,
                Level::TRACE => write!(writer, "{}", " TRC ".dimmed())?,
            }
            write!(writer, "  ")?;

            // bold white message
            if let Some(ref msg) = fields.message {
                write!(writer, "{}", msg.bold().white())?;
            }

            // colored key=value pairs
            for (k, v) in &fields.pairs {
                write!(writer, "  ")?;
                write!(writer, "{}{}{}", k.dimmed(), "=".dimmed(), v.bright_cyan())?;
            }
        } else {
            // plain text (file output)
            write!(writer, "{hh:02}:{mm:02}:{ss:02} {:<5}", level)?;
            if let Some(ref msg) = fields.message {
                write!(writer, "  {msg}")?;
            }
            for (k, v) in &fields.pairs {
                write!(writer, "  {k}={v}")?;
            }
        }

        writeln!(writer)
    }
}

// ── public init ───────────────────────────────────────────────────────────────

pub fn init_daemon_logging(log_path: &Path) -> Result<()> {
    use std::io::IsTerminal as _;

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("mosaico=info"));

    if std::io::stdout().is_terminal() {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let stdout_layer = layer()
            .event_format(DaemonFormatter)
            .with_ansi(true)
            .with_writer(std::io::stdout);

        let file_layer = layer()
            .event_format(DaemonFormatter)
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(file));

        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
    } else {
        let stdout_layer = layer()
            .event_format(DaemonFormatter)
            .with_ansi(false)
            .with_writer(std::io::stdout);

        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .init();
    }

    Ok(())
}
