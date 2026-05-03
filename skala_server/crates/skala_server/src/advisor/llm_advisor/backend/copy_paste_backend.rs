use std::fmt::{Debug, Write};
use std::sync::Arc;

use anyhow::{Context, anyhow};
use arboard::Clipboard;
use indoc::writedoc;
use log::warn;
use notify_rust::Notification;
use serde::Deserialize;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::advisor::llm_advisor::Schemas;
use crate::{Result, advisor::llm_advisor::PromptInfo};

#[derive(Clone)]
pub struct CopyPasteBackend {
    // NOTE: This really shouldn't be an Arc-Mutex, but cloning is required elsewhere
    // and this works well enough.
    // TODO(kcza): fis this.
    clipboard: Arc<Mutex<Clipboard>>,
    schemas: Schemas,
}

impl CopyPasteBackend {
    pub fn new() -> Result<Self> {
        let clipboard = Arc::new(Mutex::new(Clipboard::new()?));
        let schemas = Schemas::new();
        Ok(Self { clipboard, schemas })
    }

    pub(crate) async fn fetch<T: for<'de> Deserialize<'de>>(
        &self,
        prompt_info: impl IntoIterator<Item = PromptInfo<'_>> + Send,
    ) -> Result<T> {
        let mut prompt = prompt_info
            .into_iter()
            .map(|info| info.summary())
            .collect::<Vec<_>>()
            .join("\n");
        prompt.push('\n');
        writedoc!(
            &mut prompt,
            "
                # Output

                You MUST reply using the following schema:
                ```json
                {}
                ```
            ",
            serde_json::to_string(self.schemas.advice_response()).unwrap(),
        )
        .unwrap();

        self.clipboard.lock().await.set_text(prompt)?;
        let response = {
            let mut stdin = BufReader::new(io::stdin());
            let mut buf = String::new();
            let mut remaining_attempts = 10;
            loop {
                if remaining_attempts == 0 {
                    return Err(anyhow!("too many attempts").into());
                }
                buf.clear();

                Notification::new()
                    .summary("Skala action required")
                    .body("Paste prompt into LLM and report back")
                    .icon("gnome-terminal")
                    .show()?;

                eprint!("Prompt copied to clipboard, please paste 1-line response> ");
                stdin
                    .read_line(&mut buf)
                    .await
                    .with_context(|| anyhow!("cannot read line to buffer"))?;
                eprintln!();

                match serde_json::from_str(&buf) {
                    Ok(response) => break response,
                    Err(err) => {
                        warn!("{err}");
                        warn!(
                            "prompt ignored, please re-enter (attempts remaining {remaining_attempts})"
                        );
                        remaining_attempts -= 1;
                    }
                }
            }
        };
        Ok(response)
    }
}

impl Debug for CopyPasteBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CopyPasteBackend")
            .field("clipboard", &"_")
            .finish()
    }
}
