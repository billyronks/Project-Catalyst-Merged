//! Video module for voice-video-calling service

pub mod webrtc;
pub mod sip_video;
pub mod conference;
pub mod screen_share;
pub mod recording;
pub mod transcoding;
pub mod layout;

pub use brivas_video_sdk::{
    Conference, ConferenceSettings, Participant, ParticipantRole,
    VideoQuality, QualityMetrics,
    IceCandidate, SdpOffer, SdpAnswer, WebRtcSession,
    VideoCodec, AudioCodec,
};
