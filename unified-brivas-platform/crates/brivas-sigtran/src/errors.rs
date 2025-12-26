//! Error types for the SIGTRAN stack

use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, SigtranError>;

/// Top-level SIGTRAN error
#[derive(Debug, Error)]
pub enum SigtranError {
    #[error("SCTP error: {0}")]
    Sctp(#[from] SctpError),

    #[error("M3UA error: {0}")]
    M3ua(#[from] M3uaError),

    #[error("SCCP error: {0}")]
    Sccp(#[from] SccpError),

    #[error("TCAP error: {0}")]
    Tcap(#[from] TcapError),

    #[error("MAP error: {0}")]
    Map(#[from] MapError),

    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout")]
    Timeout,

    #[error("Connection closed")]
    ConnectionClosed,
}

/// SCTP layer errors
#[derive(Debug, Error)]
pub enum SctpError {
    #[error("Association failed: {0}")]
    AssociationFailed(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Receive failed: {0}")]
    ReceiveFailed(String),

    #[error("Heartbeat timeout")]
    HeartbeatTimeout,

    #[error("Invalid state: expected {expected:?}, got {actual:?}")]
    InvalidState {
        expected: String,
        actual: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// M3UA layer errors
#[derive(Debug, Error)]
pub enum M3uaError {
    #[error("ASP state error: {0}")]
    AspStateError(String),

    #[error("Routing error: no route to DPC {0}")]
    NoRoute(u32),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Protocol error: {0}")]
    ProtocolError(u32),

    #[error("SCTP error: {0}")]
    Sctp(#[from] SctpError),
}

/// SCCP layer errors
#[derive(Debug, Error)]
pub enum SccpError {
    #[error("Address error: {0}")]
    AddressError(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("No translation for address")]
    NoTranslation,

    #[error("Subsystem failure: SSN {0}")]
    SubsystemFailure(u8),

    #[error("Network congestion")]
    NetworkCongestion,

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("M3UA error: {0}")]
    M3ua(#[from] M3uaError),
}

/// TCAP layer errors
#[derive(Debug, Error)]
pub enum TcapError {
    #[error("Transaction not found: {0:?}")]
    TransactionNotFound(Vec<u8>),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("ASN.1 encoding error: {0}")]
    Asn1Error(String),

    #[error("Dialogue error: {0}")]
    DialogueError(String),

    #[error("Component error: {0}")]
    ComponentError(String),

    #[error("Abort received: {0:?}")]
    Abort(AbortCause),

    #[error("SCCP error: {0}")]
    Sccp(#[from] SccpError),
}

/// TCAP Abort cause
#[derive(Debug, Clone)]
pub enum AbortCause {
    UnrecognizedMessageType,
    UnrecognizedTransactionId,
    BadlyFormattedTransactionPortion,
    IncorrectTransactionPortion,
    ResourceLimitation,
    User(Vec<u8>),
}

/// MAP layer errors
#[derive(Debug, Error)]
pub enum MapError {
    #[error("Operation error: {code}")]
    OperationError {
        code: i32,
        parameter: Option<Vec<u8>>,
    },

    #[error("User error: {0}")]
    UserError(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Unknown subscriber")]
    UnknownSubscriber,

    #[error("Absent subscriber: {0}")]
    AbsentSubscriber(String),

    #[error("Facility not supported")]
    FacilityNotSupported,

    #[error("System failure")]
    SystemFailure,

    #[error("TCAP error: {0}")]
    Tcap(#[from] TcapError),

    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),
}

/// Encoding errors
#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Unsupported DCS: 0x{0:02X}")]
    UnsupportedDcs(u8),

    #[error("Invalid GSM7 character: {0}")]
    InvalidGsm7Char(char),

    #[error("Buffer too short")]
    BufferTooShort,

    #[error("Invalid BCD digit")]
    InvalidBcd,

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
