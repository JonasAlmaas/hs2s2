use std::iter::Peekable;
use std::str::Chars;

use crate::parser::KvParseError;

#[derive(Debug, PartialEq)]
pub enum Token {
    // `{`
    LeftCurly,
    /// `}`
    RightCurly,
    /// `[`
    LeftSquare,
    /// `]`
    RightSquare,
    /// `,`
    Comma,
    /// `=`
    Equal,

    /// `<!--` ... `-->`
    Comment(String),

    /// `"hello"`
    String(String),
    /// `3`
    Int(i64),
    /// `3.14`
    Float(f64),

    Identifier(String),
}

fn get_xml_comment(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    if !matches!(it.next(), Some('<'))
        || !matches!(it.next(), Some('!'))
        || !matches!(it.next(), Some('-'))
        || !matches!(it.next(), Some('-'))
    {
        return Err(KvParseError::UnexpectedSymbol(
            "XML style comments must start with <!--".to_string(),
        ));
    }

    while let Some(c) = it.next() {
        result.push(c);

        if result.ends_with("-->") {
            return Ok(Token::Comment(
                result[..result.len() - 3].trim().to_string(),
            ));
        }
    }

    Err(KvParseError::UnterminatedComment)
}

fn get_single_line_comment(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    while let Some(&c) = it.peek() {
        match c {
            '\n' => return Ok(Token::Comment(result.trim().to_string())),
            _ => {
                result.push(c);
                it.next();
            }
        }
    }

    Ok(Token::Comment(result.trim().to_string()))
}

fn get_multi_line_comment(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut result = String::new();
    let mut prev = ' ';

    while let Some(c) = it.next() {
        match c {
            '/' if prev == '*' => {
                _ = result.pop();
                return Ok(Token::Comment(result.trim().to_string()));
            }
            _ => {
                prev = c;
                result.push(c);
            }
        }
    }
    Err(KvParseError::UnterminatedComment)
}

fn get_multiline_string(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    if !matches!(it.next(), Some('\n')) {
        return Err(KvParseError::UnexpectedSymbol(
            "Multiline string must start with \"\"\"\\n".to_string(),
        ));
    }

    while let Some(c) = it.next() {
        result.push(c);

        if result.ends_with("\n\"\"\"") {
            return Ok(Token::String(result[..result.len() - 4].to_string()));
        }
    }

    Err(KvParseError::UnterminatedString)
}

fn get_string(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    if matches!(it.peek(), Some('"')) {
        it.next();
        if matches!(it.peek(), Some('"')) {
            it.next();
            return get_multiline_string(it);
        } else {
            return Ok(Token::String(result));
        }
    }

    while let Some(c) = it.next() {
        match c {
            '"' => return Ok(Token::String(result)),
            '\\' => {
                let &c = it.peek().ok_or(KvParseError::UnterminatedString)?;
                match c {
                    '"' | '\\' => result.push(c),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    //'u' => // TODO: Handle \uHHHH (unicode)
                    _ => return Err(KvParseError::InvalidEscape),
                }
                it.next();
            }
            _ => {
                result.push(c);
            }
        }
    }

    Err(KvParseError::UnterminatedString)
}

fn get_number(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut is_int = true;
    let mut s = String::new();

    while let Some(&c) = it.peek() {
        match c {
            '0'..='9' | '+' | '-' | '.' => {
                s.push(c);
                it.next();

                if c == '.' {
                    is_int = false;
                }
            }
            _ => break,
        }
    }

    if is_int {
        Ok(Token::Int(s.parse::<i64>().map_err(|_| {
            KvParseError::UnexpectedSymbol("Expected integer".to_string())
        })?))
    } else {
        Ok(Token::Float(s.parse::<f64>().map_err(|_| {
            KvParseError::UnexpectedSymbol("Expected float".to_string())
        })?))
    }
}

fn get_identifier(it: &mut Peekable<Chars>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    while let Some(&c) = it.peek() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                result.push(c);
                it.next();
            }
            _ => break,
        }
    }

    Ok(Token::Identifier(result))
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, KvParseError> {
    let mut tokens = vec![];

    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            '{' => {
                tokens.push(Token::LeftCurly);
                it.next();
            }
            '}' => {
                tokens.push(Token::RightCurly);
                it.next();
            }
            '[' => {
                tokens.push(Token::LeftSquare);
                it.next();
            }
            ']' => {
                tokens.push(Token::RightSquare);
                it.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                it.next();
            }
            '=' => {
                tokens.push(Token::Equal);
                it.next();
            }
            '<' => tokens.push(get_xml_comment(&mut it)?),
            '/' => {
                it.next();
                let c = it.next().ok_or(KvParseError::UnterminatedComment)?;
                match c {
                    '/' => tokens.push(get_single_line_comment(&mut it)?),
                    '*' => tokens.push(get_multi_line_comment(&mut it)?),
                    _ => {
                        return Err(KvParseError::UnexpectedSymbol(format!(
                            "Expected<'/' | '*'>  Actual<'{c}'>"
                        )));
                    }
                }
            }
            '"' => {
                it.next();
                tokens.push(get_string(&mut it)?);
            }
            '0'..='9' | '+' | '-' => tokens.push(get_number(&mut it)?),
            'a'..='z' | 'A'..='Z' => tokens.push(get_identifier(&mut it)?),
            ' ' | '\t' | '\r' | '\n' => {
                it.next();
            }
            _ => {
                return Err(KvParseError::UnexpectedSymbol(format!("'{c}'")));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comma() -> Result<(), KvParseError> {
        let input = ",";
        let expected = [Token::Comma];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn all_punctuation() -> Result<(), KvParseError> {
        let input = "[{]},=";
        let expected = [
            Token::LeftSquare,
            Token::LeftCurly,
            Token::RightSquare,
            Token::RightCurly,
            Token::Comma,
            Token::Equal,
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn xml_comment() -> Result<(), KvParseError> {
        let input = r#"<!-- This is a comment -->"#;
        let expected = [Token::Comment("This is a comment".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn empty_xml_comment() -> Result<(), KvParseError> {
        let input = r#"<!---->"#;
        let expected = [Token::Comment("".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn single_line_comment() -> Result<(), KvParseError> {
        let input = "foo // bar \n  baz";
        let expected = [
            Token::Identifier("foo".to_string()),
            Token::Comment("bar".to_string()),
            Token::Identifier("baz".to_string()),
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn multi_line_comment() -> Result<(), KvParseError> {
        let input = "foo /* bar \n\n baz */  baz";
        let expected = [
            Token::Identifier("foo".to_string()),
            Token::Comment("bar \n\n baz".to_string()),
            Token::Identifier("baz".to_string()),
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn trick_end_comment() -> Result<(), KvParseError> {
        let input = r#"<!-- Test --->"#;
        let expected = [Token::Comment("Test -".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn string() -> Result<(), KvParseError> {
        let input = r#""Hello \" \r \n \t world""#;
        let expected = [Token::String("Hello \" \r \n \t world".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn empty_string() -> Result<(), KvParseError> {
        let input = r#""""#;
        let expected = [Token::String("".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn multi_line_string() -> Result<(), KvParseError> {
        let input = r#"
{
"""
Hello " \"
\t world
"""
]
"#;
        let expected = [
            Token::LeftCurly,
            Token::String("Hello \" \\\"\n\\t world".to_string()),
            Token::RightSquare,
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn identifier() -> Result<(), KvParseError> {
        let input = "name";
        let expected = [Token::Identifier("name".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn identifier_no_space() -> Result<(), KvParseError> {
        let input = "name=2";
        let expected = [
            Token::Identifier("name".to_string()),
            Token::Equal,
            Token::Int(2),
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn integer() -> Result<(), KvParseError> {
        let input = "1234";
        let expected = [Token::Int(1234)];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn negative_integer() -> Result<(), KvParseError> {
        let input = "-1234";
        let expected = [Token::Int(-1234)];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn float() -> Result<(), KvParseError> {
        let input = "3.14159";
        let expected = [Token::Float(3.14159)];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn negative_float() -> Result<(), KvParseError> {
        let input = "-3.14159";
        let expected = [Token::Float(-3.14159)];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn complex() -> Result<(), KvParseError> {
        let input = r#"{ name=["1", 3.14159 , {}  ] }"#;
        let expected = [
            Token::LeftCurly,
            Token::Identifier("name".to_string()),
            Token::Equal,
            Token::LeftSquare,
            Token::String("1".to_string()),
            Token::Comma,
            Token::Float(3.14159),
            Token::Comma,
            Token::LeftCurly,
            Token::RightCurly,
            Token::RightSquare,
            Token::RightCurly,
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
