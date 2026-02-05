use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

pub struct BackupManager {
    config_path: PathBuf,
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(config_path: &Path) -> Result<Self> {
        let backup_dir = config_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("backups");

        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .with_context(|| format!("Failed to create backup dir: {}", backup_dir.display()))?;
        }

        Ok(Self {
            config_path: config_path.to_path_buf(),
            backup_dir,
        })
    }

    pub fn create_backup(&self) -> Result<PathBuf> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("config_{}.yml", timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        fs::copy(&self.config_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;

        Ok(backup_path)
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yml") {
                backups.push(path);
            }
        }

        backups.sort_by(|a, b| b.cmp(a));
        Ok(backups)
    }

    pub fn restore(&self, backup_path: &Path) -> Result<()> {
        self.create_backup()?;
        fs::copy(backup_path, &self.config_path)?;
        Ok(())
    }

    pub fn cleanup(&self, keep: usize) -> Result<usize> {
        let backups = self.list_backups()?;
        let mut removed = 0;

        for backup in backups.iter().skip(keep) {
            fs::remove_file(backup)?;
            removed += 1;
        }

        Ok(removed)
    }
}
