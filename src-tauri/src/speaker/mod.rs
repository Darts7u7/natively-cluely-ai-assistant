use anyhow::Result;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{SpeakerInput as PlatformSpeakerInput, SpeakerStream as PlatformSpeakerStream};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{SpeakerInput as PlatformSpeakerInput, SpeakerStream as PlatformSpeakerStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub struct SpeakerInput {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    inner: PlatformSpeakerInput,
}

impl SpeakerInput {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn new() -> Result<Self> {
        let inner = PlatformSpeakerInput::new(None)?;
        Ok(Self { inner })
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn stream(self) -> SpeakerStream {
        let inner = self.inner.stream();
        SpeakerStream { inner }
    }
}

pub struct SpeakerStream {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    inner: PlatformSpeakerStream,
}

impl Stream for SpeakerStream {
    type Item = f32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            Pin::new(&mut self.inner).poll_next(cx)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        std::task::Poll::Pending
    }
}
