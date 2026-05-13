mod kv3_text;

use crate::{KvFormat, KvObject};

#[derive(thiserror::Error, Debug)]
pub enum KvParseError {
    /// Tokenizer errors
    #[error("Syntax error: Unexpected token")]
    UnexpectedToken,
    #[error("Syntax error: Unterminated comment")]
    UnterminatedComment,
    #[error("Syntax error: Unterminated string")]
    UnterminatedString,
    #[error("Syntax error: Invalid escape")]
    InvalidEscape,
}

pub fn parse_kv(input: &str, format: Option<KvFormat>) -> Result<KvObject, KvParseError> {
    let format = match format {
        Some(fmt) => fmt,
        None => todo!("Automatic format detection is not supported yet"), // TODO
    };

    match format {
        KvFormat::Kv3Text => kv3_text::parse_kv3_text(input),
    }
}
