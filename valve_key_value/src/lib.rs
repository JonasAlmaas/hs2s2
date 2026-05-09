pub mod kv3;
mod serializer;

use std::io::Write;

use indexmap::IndexMap;

use crate::kv3::Kv3Id;
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

#[derive(Debug, Clone)]
pub struct KvHeader {
    pub encoding: Kv3Id,
    pub format: Kv3Id,
}

impl KvObject {
    pub fn serialize(
        &self,
        out: &mut impl Write,
        serilization_format: KvSerilizationFormat,
        header: Option<&KvHeader>,
    ) -> Result<(), std::io::Error> {
        match serilization_format {
            KvSerilizationFormat::Kv3Text => {
                /*if !matches!(self, KvObject::Map(_)) {
                }*/

                let (encoding, format) = if let Some(header) = header {
                    (&header.encoding, &header.format)
                } else {
                    (
                        &Kv3Id {
                            name: "text".to_string(),
                            id: kv3::ENC_EXT,
                        },
                        &Kv3Id {
                            name: "generic".to_string(),
                            id: kv3::FMT_GENERIC,
                        },
                    )
                };

                out.write_fmt(format_args!(
                    "<!-- kv3 encoding:{encoding} format:{format} -->"
                ))?;
                out.write_all(kv3_text::NEW_LINE)?;
                kv3_text::serialize_object(out, self, 0)?;
                out.write_all(kv3_text::NEW_LINE)?;

                Ok(())
            }
        }
    }
}
