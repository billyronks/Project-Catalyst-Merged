//! Video Transcoding
//!
//! Handles video codec transcoding for interoperability.

use brivas_video_sdk::{VideoCodec, AudioCodec};

/// Transcoding configuration
pub struct TranscodingConfig {
    pub source_video_codec: VideoCodec,
    pub target_video_codec: VideoCodec,
    pub source_audio_codec: AudioCodec,
    pub target_audio_codec: AudioCodec,
    pub target_bitrate_kbps: u32,
    pub target_resolution: (u32, u32),
    pub target_framerate: u32,
}

/// Transcoder for video/audio stream conversion
pub struct Transcoder {
    /// rtpengine control socket
    rtpengine_socket: String,
}

impl Transcoder {
    pub fn new(rtpengine_socket: &str) -> Self {
        Self {
            rtpengine_socket: rtpengine_socket.to_string(),
        }
    }

    /// Check if transcoding is needed between two codecs
    pub fn needs_transcoding(
        source: VideoCodec,
        target: VideoCodec,
    ) -> bool {
        source != target
    }

    /// Request transcoding from rtpengine
    pub async fn start_transcoding(
        &self,
        config: TranscodingConfig,
    ) -> Result<TranscodingSession, TranscodingError> {
        tracing::info!(
            source = ?config.source_video_codec,
            target = ?config.target_video_codec,
            "Starting transcoding"
        );

        // TODO: Send command to rtpengine
        Ok(TranscodingSession {
            id: uuid::Uuid::new_v4(),
            config,
            state: TranscodingState::Active,
        })
    }

    /// Stop transcoding session
    pub async fn stop_transcoding(
        &self,
        session_id: uuid::Uuid,
    ) -> Result<(), TranscodingError> {
        tracing::info!(session_id = %session_id, "Stopping transcoding");
        Ok(())
    }
}

pub struct TranscodingSession {
    pub id: uuid::Uuid,
    pub config: TranscodingConfig,
    pub state: TranscodingState,
}

#[derive(Debug, Clone, Copy)]
pub enum TranscodingState {
    Active,
    Paused,
    Stopped,
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum TranscodingError {
    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),
    #[error("rtpengine error: {0}")]
    RtpEngineError(String),
}
