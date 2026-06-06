use std::fmt;
use std::panic;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{self, Write};

use tracing::Event;
use time::OffsetDateTime;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::{self as subscriber_fmt, FmtContext, FormatEvent, FormatFields};

use crate::error::AppResult;
use crate::config::AppConfig;

struct CustomTimer;

impl FormatTime for CustomTimer {
    fn format_time(&self, w: &mut subscriber_fmt::format::Writer<'_>) -> fmt::Result {
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        write!(
            w,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02},{:03}",
            now.year(),
            now.month() as u8,
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            now.millisecond()
        )
    }
}

struct MessageExtractor<'a>(&'a mut String);

impl tracing::field::Visit for MessageExtractor<'_> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?}", value));
        }
    }
}

struct AgentFormatter;

impl<S, N> FormatEvent<S, N> for AgentFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: subscriber_fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        CustomTimer.format_time(&mut writer)?;
        write!(writer, ":{}:", event.metadata().level())?;

        let mut message = String::new();
        event.record(&mut MessageExtractor(&mut message));
        write!(writer, "{}", message)?;

        writeln!(writer)
    }
}

fn today_str() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}

struct DailyRollingFile {
    dir: PathBuf,
    prefix: String,
    date: String,
    file: Option<std::fs::File>,
}

impl DailyRollingFile {
    fn new(dir: PathBuf, prefix: &str) -> Self {
        Self {
            dir,
            prefix: prefix.to_string(),
            date: String::new(),
            file: None,
        }
    }

    fn rotate(&mut self) -> io::Result<()> {
        let today = today_str();
        if self.date == today {
            return Ok(());
        }
        self.date = today;
        let path = self.dir.join(format!("{}_{}.log", self.date, self.prefix));
        self.file = Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?,
        );
        Ok(())
    }
}

impl Write for DailyRollingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.rotate()?;
        self.file.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.as_mut().map_or(Ok(()), |f| f.flush())
    }
}

pub fn init(config: &AppConfig) -> AppResult<WorkerGuard> {
    std::fs::create_dir_all(&config.log_dir)?;

    let file_appender = DailyRollingFile::new(config.log_dir.clone(), "agent");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let stdout_layer = subscriber_fmt::Layer::default()
        .event_format(AgentFormatter)
        .with_writer(std::io::stdout);

    let file_layer = subscriber_fmt::Layer::default()
        .event_format(AgentFormatter)
        .with_writer(non_blocking_file)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let (file, line) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line()))
            .unwrap_or(("<unknown>", 0));
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown panic payload".to_string());

        tracing::error!("程序崩溃 ({}:{}): {}", file, line, payload);
        prev_hook(panic_info);
    }));

    Ok(guard)
}
