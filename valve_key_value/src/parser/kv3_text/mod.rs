mod tokenizer;

use crate::KvObject;
use crate::parser::KvParseError;
use crate::parser::kv3_text::tokenizer::tokenize;

pub fn parse_kv3_text(input: &str) -> Result<KvObject, KvParseError> {
    _ = tokenize(input)?;

    Ok(KvObject::Null)
}
