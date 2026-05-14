mod tokenizer;

use std::iter::Peekable;
use std::slice::Iter;

use indexmap::IndexMap;

use crate::KvObject;
use crate::parser::KvParseError;
use crate::parser::kv3_text::tokenizer::{Token, tokenize};

fn consume_comments(it: &mut Peekable<Iter<Token>>) {
    while let Some(t) = it.peek() {
        match t {
            Token::Comment(_) => _ = it.next(),
            _ => return,
        }
    }
}

fn parse_map(it: &mut Peekable<Iter<Token>>) -> Result<KvObject, KvParseError> {
    let mut result = IndexMap::new();

    consume_comments(it);

    while let Some(t) = it.next() {
        match t {
            Token::RightCurly => return Ok(KvObject::Map(result)),
            Token::Identifier(id) => {
                consume_comments(it);
                let t = it.next().ok_or(KvParseError::EarlyEnd)?;
                if matches!(t, Token::Equal) {
                    if result.contains_key(id) {
                        return Err(KvParseError::DuplicateKey(id.clone()));
                    }
                    result.insert(id.clone(), parse_obj(it)?);
                } else {
                    return Err(KvParseError::UnexpectedToken(format!(
                        "Expected<=>  Actual<{t:?}>"
                    )));
                }
            }
            Token::Comment(_) => unreachable!(),
            _ => {
                return Err(KvParseError::UnexpectedToken(format!(
                    "Expected<'}}' | Identifier>  Actual<{t:?}>"
                )));
            }
        }

        consume_comments(it);
    }

    Err(KvParseError::UnterminatedMap)
}

fn parse_array(it: &mut Peekable<Iter<Token>>) -> Result<KvObject, KvParseError> {
    let mut result = Vec::new();

    while matches!(it.peek(), Some(_)) {
        consume_comments(it);

        if matches!(it.peek(), Some(Token::RightSquare)) {
            it.next();
            return Ok(KvObject::Array(result));
        }

        result.push(parse_obj(it)?);

        consume_comments(it);
        if matches!(it.peek(), Some(Token::Comma)) {
            _ = it.next();
        }
    }

    Err(KvParseError::UnterminatedArray)
}

fn parse_obj(it: &mut Peekable<Iter<Token>>) -> Result<KvObject, KvParseError> {
    consume_comments(it);

    let t = it.next().ok_or(KvParseError::EarlyEnd)?;
    match t {
        Token::LeftCurly => parse_map(it),
        Token::LeftSquare => parse_array(it),
        Token::String(v) => Ok(KvObject::String(v.clone())),
        Token::Int(v) => Ok(KvObject::Int(*v)),
        Token::Float(v) => Ok(KvObject::Float(*v)),
        Token::Identifier(id) => match id.as_str() {
            "true" => Ok(KvObject::Bool(true)),
            "false" => Ok(KvObject::Bool(false)),
            _ => Err(KvParseError::UnexpectedToken(format!(
                "Expected<true | false>  Actual<{id}>"
            ))),
        },
        Token::RightCurly | Token::RightSquare | Token::Comma | Token::Equal => {
            Err(KvParseError::UnexpectedToken(format!("{t:?}")))
        }
        Token::Comment(_) => unreachable!(),
    }
}

pub fn parse_kv3_text(input: &str) -> Result<KvObject, KvParseError> {
    let tokens = tokenize(input)?;
    let mut it = tokens.iter().peekable();
    parse_obj(&mut it)
}

#[cfg(test)]
mod tests {
    use indexmap::indexmap;

    use super::*;

    #[test]
    fn parse_empty_object() -> Result<(), KvParseError> {
        let input = "{}";
        let expected = KvObject::Map(IndexMap::new());

        let actual = parse_kv3_text(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_empty_array() -> Result<(), KvParseError> {
        let input = "[]";
        let expected = KvObject::Array(Vec::new());

        let actual = parse_kv3_text(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_simple_object() -> Result<(), KvParseError> {
        let input = r#"
{
    foo = 1
    bar = 3.14
    baz = "my string"
}
"#;
        let expected = KvObject::Map(indexmap! {
            "foo".to_string() => KvObject::Int(1),
            "bar".to_string() => KvObject::Float(3.14),
            "baz".to_string() => KvObject::String("my string".to_string()),
        });

        let actual = parse_kv3_text(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_simple_array() -> Result<(), KvParseError> {
        let input = "[ 1, 2, 3 ]";
        let expected = KvObject::Array(vec![KvObject::Int(1), KvObject::Int(2), KvObject::Int(3)]);

        let actual = parse_kv3_text(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_map_with_array() -> Result<(), KvParseError> {
        let input = "{ foo = [ 1, 2, 3 ] }";
        let expected = KvObject::Map(indexmap! {
            "foo".to_string() => KvObject::Array(vec![
                KvObject::Int(1),
                KvObject::Int(2),
                KvObject::Int(3),
            ])
        });

        let actual = parse_kv3_text(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_complex_map() -> Result<(), KvParseError> {
        let input = r#"
<!-- Header -->
{
    // This is a comment
    foo = true
    bar=false
    baz = "my string"
    aa =
    {
        foo = 12
        bar = 3.14
    }
    bb = /* Inner comment */ [ 0, 1, 2 ]
    cc =
    [
        {
            inner = true
        }
    ]
}
"#;
        let expected = KvObject::Map(indexmap! {
            "foo".to_string() => KvObject::Bool(true),
            "bar".to_string() => KvObject::Bool(false),
            "baz".to_string() => KvObject::String("my string".to_string()),
            "aa".to_string() => KvObject::Map(indexmap! {
                "foo".to_string() => KvObject::Int(12),
                "bar".to_string() => KvObject::Float(3.14),
            }),
            "bb".to_string() => KvObject::Array(vec![
                KvObject::Int(0),
                KvObject::Int(1),
                KvObject::Int(2),
            ]),
            "cc".to_string() => KvObject::Array(vec![
                KvObject::Map(indexmap! {
                    "inner".to_string() => KvObject::Bool(true)
                })
            ]),
        });

        let actual = parse_kv3_text(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
