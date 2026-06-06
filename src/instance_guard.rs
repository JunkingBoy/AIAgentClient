use std::fs::{File, OpenOptions};
use std::path::Path;

use fs2::FileExt;

use crate::error::AppError;

pub trait InstanceGuard: Send {
    fn try_lock(lock_path: &Path) -> Result<Self, AppError>
    where
        Self: Sized;
}

pub struct FileLockGuard {
    _file: File,
}

impl InstanceGuard for FileLockGuard {
    fn try_lock(lock_path: &Path) -> Result<Self, AppError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|e| {
                let msg = format!(
                    "无法创建或打开锁文件 '{}': {}",
                    lock_path.display(),
                    e
                );
                AppError::InstanceExists(msg)
            })?;

        file.try_lock_exclusive().map_err(|_| {
            let msg = format!(
                "检测到已有实例在运行（锁文件: {}），请勿重复启动",
                lock_path.display()
            );
            AppError::InstanceExists(msg)
        })?;

        Ok(Self { _file: file })
    }
}
