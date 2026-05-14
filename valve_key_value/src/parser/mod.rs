mod kv3_text;

use crate::{KvFormat, KvObject};

#[derive(thiserror::Error, Debug)]
pub enum KvParseError {
    // Tokenizer
    #[error("Early end")]
    EarlyEnd,
    #[error("Unexpected symbol: {0}")]
    UnexpectedSymbol(String),
    #[error("Unterminated comment")]
    UnterminatedComment,
    #[error("Unterminated string")]
    UnterminatedString,
    #[error("Invalid escape")]
    InvalidEscape,

    // Parser
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Unterminated map")]
    UnterminatedMap,
    #[error("Unterminated array")]
    UnterminatedArray,
    #[error("Duplicate key in map \"{0}\"")]
    DuplicateKey(String),
}

pub fn parse_kv(input: &str, format: Option<KvFormat>) -> Result<KvObject, KvParseError> {
    let format = match format {
        Some(fmt) => fmt,
        None => todo!("Automatic format detection is not supported yet"),
    };

    match format {
        KvFormat::Kv3Text => kv3_text::parse_kv3_text(input),
    }
}
