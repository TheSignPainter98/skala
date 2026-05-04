use std::fmt::{Debug, Write};
use std::sync::Arc;

use anyhow::{Context, anyhow};
use arboard::Clipboard;
use indoc::writedoc;
use log::{info, warn};
use notify_rust::Notification;
use serde::Deserialize;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::advisor::llm_advisor::Schemas;
use crate::advisor::llm_advisor::backend::editor::Editor;
use crate::{Result, advisor::llm_advisor::PromptInfo};

#[derive(Clone)]
pub struct CopyPasteBackend {
    // NOTE: This really shouldn't be an Arc-Mutex, but cloning is required elsewhere
    // and this works well enough.
    // TODO(kcza): fix this.
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
        let schema_json = serde_json::to_string(self.schemas.advice_response()).unwrap();
        writedoc!(
            &mut prompt,
            "
                # Output

                You MUST reply using the following schema:
                ```json
                {schema_json}
                ```
            ",
        )
        .unwrap();

        self.clipboard.lock().await.set_text(prompt)?;
        let response = {
            let notification_result = Notification::new()
                .summary("Reactor advice required")
                .body("Paste prompt into LLM and report back")
                .icon("gnome-terminal")
                .show();
            if let Err(err) = notification_result {
                warn!("cannot display notification: {err}");
            }

            let mut stdin = BufReader::new(io::stdin());
            let mut editor = Editor::new("nvim")?;
            loop {
                let content = editor.edit().await?;
                if content.trim().is_empty() {
                    return Err(anyhow!("aborted by user").into());
                }
                match serde_json::from_str(&content) {
                    Ok(response) => {
                        info!("received valid advice");
                        break response;
                    }
                    Err(err) => {
                        warn!("{err}");
                        stdin
                            .read_line(&mut String::new())
                            .await
                            .context("cannot read line")?;
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
