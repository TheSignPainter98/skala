use anyhow::{Context, anyhow};
use tempfile::NamedTempFile;
use tokio::{fs, process::Command};

use crate::Result;

pub struct Editor {
    command: String,
    file: NamedTempFile,
}

impl Editor {
    pub fn new(command: impl Into<String>) -> Result<Self> {
        let command = command.into();
        let file = NamedTempFile::new().context("cannot create named temp file")?;
        Ok(Self { command, file })
    }

    pub async fn edit(&mut self) -> Result<String> {
        let path = self.file.path();
        let exit_code = Command::new(&self.command)
            .arg(path)
            .status()
            .await
            .context("cannot edit temp file")?;
        if !exit_code.success() {
            return Err(anyhow!("editor exited with code {exit_code}").into());
        }
        let ret = fs::read_to_string(path)
            .await
            .context("cannot read temp file")?;
        Ok(ret)
    }
}
