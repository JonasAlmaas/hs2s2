use std::io::Write;

use crate::kv3::{self, Kv3Id, KvHeader};
use crate::{KvFormat, KvObject};

impl KvObject {
    pub fn serialize(
        &self,
        out: &mut impl Write,
        serilization_format: KvFormat,
        header: Option<&KvHeader>,
    ) -> Result<(), std::io::Error> {
        match serilization_format {
            KvFormat::Kv3Text => {
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
                out.write_all(kv3::kv3_text::NEW_LINE)?;
                kv3::kv3_text::serializer::serialize_object(out, self, 0)?;
                out.write_all(kv3::kv3_text::NEW_LINE)?;

                Ok(())
            }
        }
    }
}
