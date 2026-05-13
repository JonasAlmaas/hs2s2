pub mod kv3;
pub mod parser;
pub mod serializer;

use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub enum KvObject {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Map(IndexMap<String, KvObject>),
    Array(Vec<KvObject>),
    Bin(Vec<u8>),
}

#[derive(Debug, Clone, Copy)]
pub enum KvFormat {
    Kv3Text,
}
