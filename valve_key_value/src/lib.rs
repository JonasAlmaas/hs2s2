pub mod kv3;

use std::io::Write;

use indexmap::IndexMap;

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
    fn serialize_kv3_text(
        &self,
        indent: usize,
        out: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        match self {
            KvObject::Null => out.write_all(b" null")?,
            KvObject::String(v) => out.write_fmt(format_args!(" \"{v}\""))?,
            KvObject::Int(v) => out.write_fmt(format_args!(" {v}"))?,
            KvObject::Float(v) => out.write_fmt(format_args!(" {v}"))?,
            KvObject::Bool(v) => out.write_fmt(format_args!(" {v}"))?,
            KvObject::Map(map) => {
                let tabs = "\t".repeat(indent);

                out.write_all(b"\r\n")?;
                out.write_all(tabs.as_bytes())?;
                out.write_all(b"{\r\n")?;

                {
                    let indent = indent + 1;
                    let tabs = "\t".repeat(indent);

                    for (k, v) in map {
                        out.write_all(tabs.as_bytes())?;
                        out.write_fmt(format_args!("{} =", k))?;
                        v.serialize_kv3_text(indent, out)?;
                        out.write_all(b"\r\n")?;
                    }
                }
                out.write_all(tabs.as_bytes())?;
                out.write_all(b"}")?;
            }
            KvObject::Array(arr) => {
                let tabs = "\t".repeat(indent);
                let multiline = arr.len() > 0 && matches!(arr[0], KvObject::Map(_));

                if multiline {
                    out.write_all(b"\r\n")?;
                    out.write_all(tabs.as_bytes())?;
                } else {
                    out.write_all(b" ")?;
                }

                out.write_all(b"[")?;

                {
                    let indent = indent + 1;

                    for (ix, v) in arr.iter().enumerate() {
                        v.serialize_kv3_text(indent, out)?;

                        if ix < arr.len() - 1 {
                            out.write_all(b",")?;
                        } else if !multiline {
                            out.write_all(b" ")?;
                        }
                    }
                }

                if multiline {
                    out.write_all(b"\r\n")?;
                    out.write_all(tabs.as_bytes())?;
                }

                out.write_all(b"]")?;
            }
            KvObject::Bin(_) => todo!(),
        }

        Ok(())
    }

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

                self.serialize_kv3_text(0, out)?;
                out.write_all(b"\r\n")?;

                Ok(())
            }
        }
    }
}
