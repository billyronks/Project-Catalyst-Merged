//! Video and audio codec types

use serde::{Deserialize, Serialize};

/// Supported video codecs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    VP8,
    VP9,
    H264,
    H265,
    AV1,
}

impl VideoCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VP8 => "VP8",
            Self::VP9 => "VP9",
            Self::H264 => "H264",
            Self::H265 => "H265",
            Self::AV1 => "AV1",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::VP8 => "video/VP8",
            Self::VP9 => "video/VP9",
            Self::H264 => "video/H264",
            Self::H265 => "video/H265",
            Self::AV1 => "video/AV1",
        }
    }

    /// Get the SDP payload type for this codec
    pub fn payload_type(&self) -> u8 {
        match self {
            Self::VP8 => 96,
            Self::VP9 => 98,
            Self::H264 => 100,
            Self::H265 => 102,
            Self::AV1 => 104,
        }
    }
}

/// Supported audio codecs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCodec {
    Opus,
    G711U,
    G711A,
    G722,
}

impl AudioCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Opus => "opus",
            Self::G711U => "PCMU",
            Self::G711A => "PCMA",
            Self::G722 => "G722",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Opus => "audio/opus",
            Self::G711U => "audio/PCMU",
            Self::G711A => "audio/PCMA",
            Self::G722 => "audio/G722",
        }
    }

    pub fn sample_rate(&self) -> u32 {
        match self {
            Self::Opus => 48000,
            Self::G711U | Self::G711A => 8000,
            Self::G722 => 16000,
        }
    }

    pub fn payload_type(&self) -> u8 {
        match self {
            Self::Opus => 111,
            Self::G711U => 0,
            Self::G711A => 8,
            Self::G722 => 9,
        }
    }
}

/// Codec configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecConfig {
    pub video_codecs: Vec<VideoCodec>,
    pub audio_codecs: Vec<AudioCodec>,
    pub preferred_video_codec: VideoCodec,
    pub preferred_audio_codec: AudioCodec,
}

impl Default for CodecConfig {
    fn default() -> Self {
        Self {
            video_codecs: vec![VideoCodec::VP8, VideoCodec::VP9, VideoCodec::H264],
            audio_codecs: vec![AudioCodec::Opus, AudioCodec::G711U],
            preferred_video_codec: VideoCodec::VP8,
            preferred_audio_codec: AudioCodec::Opus,
        }
    }
}
