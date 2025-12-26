//! BRIVAS Video SDK
//!
//! Video calling types and abstractions for WebRTC and SIP video.

pub mod webrtc;
pub mod conference;
pub mod codec;
pub mod quality;

pub use webrtc::{IceCandidate, SdpOffer, SdpAnswer, WebRtcSession};
pub use conference::{Conference, ConferenceSettings, Participant, ParticipantRole};
pub use codec::{VideoCodec, AudioCodec};
pub use quality::{VideoQuality, QualityMetrics};
