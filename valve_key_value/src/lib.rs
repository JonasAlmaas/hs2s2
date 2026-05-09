pub mod kv3;
mod serializer;

use std::io::Write;

use indexmap::IndexMap;

use crate::serializer::kv3_text;

#[derive(Debug, Clone, Copy)]
pub enum KvSerilizationFormat {
    Kv3Text,
}

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

impl KvObject {
    pub fn serialize(
        &self,
        format: KvSerilizationFormat,
        out: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        match format {
            KvSerilizationFormat::Kv3Text => {
                /*if !matches!(self, KvObject::Map(_)) {
                }*/

                out.write_fmt(format_args!(
                    "<!-- kv3 {} {} -->",
                    kv3::Kv3Id {
                        name: "encoding:text".to_string(),
                        id: kv3::ENC_EXT,
                    },
                    kv3::Kv3Id {
                        name: "format:generic".to_string(),
                        id: kv3::FMT_GENERIC,
                    },
                ))?;

                kv3_text::serialize_object(self, 0, out)?;
                out.write_all(b"\r\n")?;

                Ok(())
            }
        }
    }
}
