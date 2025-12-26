//! Video quality types

use serde::{Deserialize, Serialize};

/// Video quality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoQuality {
    /// Automatic (adaptive)
    Auto,
    /// 320x240 @ 15fps, 150kbps
    Low,
    /// 640x480 @ 24fps, 500kbps
    Medium,
    /// 1280x720 @ 30fps, 1.5Mbps
    High,
    /// 1920x1080 @ 30fps, 4Mbps
    Hd,
    /// 3840x2160 @ 30fps, 15Mbps
    UltraHd,
}

impl VideoQuality {
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::Auto => (1280, 720),
            Self::Low => (320, 240),
            Self::Medium => (640, 480),
            Self::High => (1280, 720),
            Self::Hd => (1920, 1080),
            Self::UltraHd => (3840, 2160),
        }
    }

    pub fn framerate(&self) -> u32 {
        match self {
            Self::Auto => 30,
            Self::Low => 15,
            Self::Medium => 24,
            Self::High | Self::Hd | Self::UltraHd => 30,
        }
    }

    pub fn bitrate_kbps(&self) -> u32 {
        match self {
            Self::Auto => 1500,
            Self::Low => 150,
            Self::Medium => 500,
            Self::High => 1500,
            Self::Hd => 4000,
            Self::UltraHd => 15000,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Hd => "hd",
            Self::UltraHd => "4k",
        }
    }
}

impl Default for VideoQuality {
    fn default() -> Self {
        Self::Auto
    }
}

/// Quality metrics for a video stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Video bitrate in kbps
    pub video_bitrate_kbps: u32,
    /// Video framerate
    pub video_framerate: f32,
    /// Current video resolution
    pub video_resolution: String,
    /// Packets lost
    pub video_packets_lost: u32,
    /// Jitter in milliseconds
    pub video_jitter_ms: f32,
    
    /// Audio bitrate in kbps
    pub audio_bitrate_kbps: u32,
    /// Audio packets lost
    pub audio_packets_lost: u32,
    /// Audio jitter in milliseconds
    pub audio_jitter_ms: f32,
    
    /// Round-trip time in milliseconds
    pub rtt_ms: f32,
    /// Available bandwidth in kbps
    pub available_bandwidth_kbps: u32,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            video_bitrate_kbps: 0,
            video_framerate: 0.0,
            video_resolution: "0x0".to_string(),
            video_packets_lost: 0,
            video_jitter_ms: 0.0,
            audio_bitrate_kbps: 0,
            audio_packets_lost: 0,
            audio_jitter_ms: 0.0,
            rtt_ms: 0.0,
            available_bandwidth_kbps: 0,
        }
    }
}
