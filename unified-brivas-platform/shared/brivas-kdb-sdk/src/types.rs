//! kdb+ data types

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// kdb+ atom types
#[derive(Debug, Clone, PartialEq)]
pub enum KdbAtom {
    Boolean(bool),
    Guid(Uuid),
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Real(f32),
    Float(f64),
    Char(char),
    Symbol(String),
    Timestamp(DateTime<Utc>),
    Date(NaiveDate),
    Time(NaiveTime),
    Null,
}

/// kdb+ list types
#[derive(Debug, Clone)]
pub enum KdbList {
    Booleans(Vec<bool>),
    Guids(Vec<Uuid>),
    Bytes(Vec<u8>),
    Shorts(Vec<i16>),
    Ints(Vec<i32>),
    Longs(Vec<i64>),
    Reals(Vec<f32>),
    Floats(Vec<f64>),
    Chars(String),
    Symbols(Vec<String>),
    Timestamps(Vec<DateTime<Utc>>),
    Dates(Vec<NaiveDate>),
    Times(Vec<NaiveTime>),
    Mixed(Vec<KdbValue>),
}

/// kdb+ dictionary
#[derive(Debug, Clone)]
pub struct KdbDict {
    pub keys: Box<KdbValue>,
    pub values: Box<KdbValue>,
}

/// kdb+ table
#[derive(Debug, Clone)]
pub struct KdbTable {
    pub columns: Vec<String>,
    pub data: Vec<KdbList>,
}

/// Any kdb+ value
#[derive(Debug, Clone)]
pub enum KdbValue {
    Atom(KdbAtom),
    List(KdbList),
    Dict(KdbDict),
    Table(KdbTable),
    Error(String),
}

/// CDR record for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cdr {
    pub call_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source_number: String,
    pub destination_number: String,
    pub carrier_id: Uuid,
    pub duration_secs: i64,
    pub disposition: String,
    pub pdd_ms: i64,
    pub rate: f64,
    pub cost: f64,
}

/// Traffic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub total_calls: i64,
    pub active_calls: i64,
    pub calls_per_second: f64,
    pub avg_duration: f64,
    pub total_minutes: f64,
}

/// Carrier statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarrierStats {
    pub carrier_id: Uuid,
    pub carrier_name: String,
    pub total_calls: i64,
    pub successful_calls: i64,
    pub failed_calls: i64,
    pub asr: f64,
    pub acd: f64,
    pub pdd: f64,
}

/// QoS metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosMetrics {
    pub carrier_id: Uuid,
    pub pdd_avg: f64,
    pub pdd_p95: f64,
    pub jitter_avg: f64,
    pub packet_loss: f64,
    pub mos_avg: f64,
}
