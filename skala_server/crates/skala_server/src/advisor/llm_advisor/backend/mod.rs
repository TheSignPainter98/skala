mod copy_paste_backend;
mod openai_backend;

pub use self::copy_paste_backend::CopyPasteBackend;
pub use self::openai_backend::OpenAiBackend;

use std::fmt::Debug;

use serde::Deserialize;

use crate::Result;
use crate::advisor::llm_advisor::PromptInfo;

#[derive(Clone, Debug)]
pub enum Backend {
    CopyPaste(CopyPasteBackend),
    OpenAi(Box<OpenAiBackend>),
}

impl Backend {
    pub(crate) async fn fetch<T: for<'de> Deserialize<'de>>(
        &self,
        prompt_info: impl IntoIterator<Item = PromptInfo<'_>> + Send,
    ) -> Result<T> {
        match self {
            Self::CopyPaste(inner) => inner.fetch(prompt_info).await,
            Self::OpenAi(inner) => inner.fetch(prompt_info).await,
        }
    }
}

impl From<CopyPasteBackend> for Backend {
    fn from(inner: CopyPasteBackend) -> Self {
        Self::CopyPaste(inner)
    }
}

impl From<OpenAiBackend> for Backend {
    fn from(inner: OpenAiBackend) -> Self {
        Self::OpenAi(Box::new(inner))
    }
}
