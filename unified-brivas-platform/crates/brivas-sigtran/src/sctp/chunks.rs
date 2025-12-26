//! SCTP Chunk encoding/decoding

use bytes::{Bytes, BytesMut, Buf, BufMut};
use super::ChunkType;

/// SCTP Common Header
#[derive(Debug, Clone)]
pub struct SctpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub verification_tag: u32,
    pub checksum: u32,
}

impl SctpHeader {
    pub const SIZE: usize = 12;

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.source_port);
        buf.put_u16(self.destination_port);
        buf.put_u32(self.verification_tag);
        buf.put_u32(self.checksum);
    }

    pub fn decode(buf: &mut Bytes) -> Option<Self> {
        if buf.remaining() < Self::SIZE {
            return None;
        }
        Some(Self {
            source_port: buf.get_u16(),
            destination_port: buf.get_u16(),
            verification_tag: buf.get_u32(),
            checksum: buf.get_u32(),
        })
    }
}

/// SCTP Chunk Header
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    pub chunk_type: u8,
    pub flags: u8,
    pub length: u16,
}

impl ChunkHeader {
    pub const SIZE: usize = 4;

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(self.chunk_type);
        buf.put_u8(self.flags);
        buf.put_u16(self.length);
    }

    pub fn decode(buf: &mut Bytes) -> Option<Self> {
        if buf.remaining() < Self::SIZE {
            return None;
        }
        Some(Self {
            chunk_type: buf.get_u8(),
            flags: buf.get_u8(),
            length: buf.get_u16(),
        })
    }
}

/// DATA Chunk
#[derive(Debug, Clone)]
pub struct DataChunk {
    pub tsn: u32,
    pub stream_id: u16,
    pub stream_seq: u16,
    pub ppid: u32,
    pub user_data: Bytes,
    pub unordered: bool,
    pub beginning: bool,
    pub ending: bool,
}

impl DataChunk {
    /// Calculate flags byte
    pub fn flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.unordered {
            flags |= 0x04;
        }
        if self.beginning {
            flags |= 0x02;
        }
        if self.ending {
            flags |= 0x01;
        }
        flags
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        let length = 16 + self.user_data.len() as u16;
        let header = ChunkHeader {
            chunk_type: ChunkType::Data as u8,
            flags: self.flags(),
            length,
        };
        header.encode(buf);
        
        buf.put_u32(self.tsn);
        buf.put_u16(self.stream_id);
        buf.put_u16(self.stream_seq);
        buf.put_u32(self.ppid);
        buf.put_slice(&self.user_data);
        
        // Pad to 4-byte boundary
        let padding = (4 - (self.user_data.len() % 4)) % 4;
        for _ in 0..padding {
            buf.put_u8(0);
        }
    }

    pub fn decode(flags: u8, mut data: Bytes) -> Option<Self> {
        if data.remaining() < 12 {
            return None;
        }
        
        let tsn = data.get_u32();
        let stream_id = data.get_u16();
        let stream_seq = data.get_u16();
        let ppid = data.get_u32();
        
        Some(Self {
            tsn,
            stream_id,
            stream_seq,
            ppid,
            user_data: data,
            unordered: (flags & 0x04) != 0,
            beginning: (flags & 0x02) != 0,
            ending: (flags & 0x01) != 0,
        })
    }
}

/// HEARTBEAT Chunk
#[derive(Debug, Clone)]
pub struct HeartbeatChunk {
    pub info: Bytes,
}

impl HeartbeatChunk {
    pub fn encode(&self, buf: &mut BytesMut) {
        let length = 4 + 4 + self.info.len() as u16;
        let header = ChunkHeader {
            chunk_type: ChunkType::Heartbeat as u8,
            flags: 0,
            length,
        };
        header.encode(buf);
        
        // Heartbeat Info parameter
        buf.put_u16(1); // Parameter type
        buf.put_u16(4 + self.info.len() as u16);
        buf.put_slice(&self.info);
        
        // Pad to 4-byte boundary
        let padding = (4 - (self.info.len() % 4)) % 4;
        for _ in 0..padding {
            buf.put_u8(0);
        }
    }
}
