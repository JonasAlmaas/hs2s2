use std::fmt::Display;

use uuid::{Uuid, uuid};

/// Generic format for arbitrary data, with no assumptions about how the data will be used
pub const FMT_GENERIC: Uuid = uuid!("7412167c-06e9-4698-aff2-e63eb59037e7");

/// Automatic binary encoding selection. Chooses the best binary encoding at write time
pub const ENC_BIN_AUTO: Uuid = uuid!("6eb109e6-6b85-4583-a312-703a6e04068c");
/// Block-compressed binary encoding using Valve's custom LZ77-style block compression.</summary>
pub const ENC_BIN_BLK_COMP: Uuid = uuid!("15191a46-151c-1f1c-171b-151c11171f12");
/// LZ4-compressed binary encoding
pub const ENC_BIN_LZ4: Uuid = uuid!("6847248a-63a1-4f5c-a197-53806fd9b119");
/// Zstandard-compressed binary encoding
pub const ENC_BIN_ZSTD: Uuid = uuid!("6f62a000-fef0-4305-a35f-042346b1db29");
/// Uncompressed binary encoding. Data is stored as-is with a string table and typed values
pub const ENC_BIN: Uuid = uuid!("1b860500-f7d8-40c1-ad82-75a48267e714");
/// Human-readable text encoding with an XML-style comment header
pub const ENC_EXT: Uuid = uuid!("e21c7f3c-8a33-41c5-9977-a76d3a32aa0d");

pub const FLAG_RESOURCE: &str = "resource";
pub const FLAG_RESOURCE_NAME: &str = "resourcename";
pub const FLAG_PANORAMA: &str = "panorama";
pub const FLAG_SOUNDEVENT: &str = "soundevent";
pub const FLAG_SUBCLASS: &str = "subclass";

#[derive(Debug, Clone)]
pub struct Kv3Id {
    pub name: String,
    pub id: Uuid,
}

impl Display for Kv3Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:version{{{}}}", self.name, self.id)
    }
}

/// Only applies to KV3
#[derive(Debug, Clone)]
pub struct KvHeader {
    pub encoding: Kv3Id,
    pub format: Kv3Id,
}
