use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;

use time::OffsetDateTime;

/// 按日滚动的日志文件
pub(crate) struct DailyRollingFile {
    dir: PathBuf,
    prefix: String,
    date: String,
    file: Option<std::fs::File>,
}

impl DailyRollingFile {
    pub(crate) fn new(dir: PathBuf, prefix: &str) -> Self {
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
        self.file = Some(OpenOptions::new().create(true).append(true).open(path)?);
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

fn today_str() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}
