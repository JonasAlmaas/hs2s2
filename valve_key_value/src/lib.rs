pub mod kv3;
pub mod parser;
pub mod serializer;

use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct KvMapEntry {
    pub value: KvObject,
    pub flag: Option<String>,
}

impl KvMapEntry {
    pub fn new(obj: KvObject) -> KvMapEntry {
        KvMapEntry {
            value: obj,
            flag: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KvObject {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Map(IndexMap<String, KvMapEntry>),
    Array(Vec<KvObject>),
    Bin(Vec<u8>),
}

#[derive(Debug, Clone, Copy)]
pub enum KvFormat {
    Kv3Text,
}
