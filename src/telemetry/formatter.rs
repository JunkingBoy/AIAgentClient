use std::fmt;

use time::OffsetDateTime;
use tracing::Event;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::registry::LookupSpan;

/// 时间格式：`2026-06-06 18:41:32,183`
pub(crate) struct CustomTimer;

impl FormatTime for CustomTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
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

/// 从 Event 中只提取 `message` 字段
pub(crate) struct MessageExtractor<'a>(pub(crate) &'a mut String);

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

/// 日志行格式：`时间:级别:消息`
pub(crate) struct AgentFormatter;

impl<S, N> FormatEvent<S, N> for AgentFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
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
