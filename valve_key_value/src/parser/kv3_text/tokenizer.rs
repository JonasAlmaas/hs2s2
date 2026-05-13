use std::iter::Peekable;
use std::str::Chars;

use crate::parser::KvParseError;

#[derive(Debug, PartialEq)]
pub enum Token {
    // `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
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

fn get_comment(it: &mut Peekable<Chars<'_>>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    if !matches!(it.peek(), Some('<')) {
        return Err(KvParseError::UnexpectedToken);
    }
    it.next();
    if !matches!(it.peek(), Some('!')) {
        return Err(KvParseError::UnexpectedToken);
    }
    it.next();
    if !matches!(it.peek(), Some('-')) {
        return Err(KvParseError::UnexpectedToken);
    }
    it.next().ok_or(KvParseError::UnexpectedToken)?;
    if !matches!(it.peek(), Some('-')) {
        return Err(KvParseError::UnexpectedToken);
    }
    it.next().ok_or(KvParseError::UnexpectedToken)?;

    while let Some(&c) = it.peek() {
        result.push(c);

        if c == '-' {
            it.next();
            let &c = it.peek().ok_or(KvParseError::UnterminatedComment)?;
            if c == '-' {
                while let Some(&c) = it.peek()
                    && c == '-'
                {
                    result.push(c);
                    it.next();
                }

                let &c = it.peek().ok_or(KvParseError::UnterminatedComment)?;
                if c == '>' {
                    result.push(c);
                    it.next();
                    return Ok(Token::Comment(
                        result[..result.len() - 3].trim().to_string(),
                    ));
                }
            }
        } else {
            it.next();
        }
    }

    Err(KvParseError::UnterminatedComment)
}

fn get_string(it: &mut Peekable<Chars<'_>>) -> Result<Token, KvParseError> {
    let mut result = String::new();

    if !matches!(it.peek(), Some('"')) {
        return Err(KvParseError::UnexpectedToken);
    }
    it.next();

    while let Some(&c) = it.peek() {
        match c {
            '\\' => {
                it.next();
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
            '"' => {
                it.next();
                return Ok(Token::String(result));
            }
            _ => {
                result.push(c);
                it.next();
            }
        }
    }

    Err(KvParseError::UnterminatedString)
}

fn get_number(it: &mut Peekable<Chars<'_>>) -> Result<Token, KvParseError> {
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
        Ok(Token::Int(
            s.parse::<i64>()
                .map_err(|_| KvParseError::UnexpectedToken)?,
        ))
    } else {
        Ok(Token::Float(
            s.parse::<f64>()
                .map_err(|_| KvParseError::UnexpectedToken)?,
        ))
    }
}

fn get_identifier(it: &mut Peekable<Chars<'_>>) -> Result<Token, KvParseError> {
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
                tokens.push(Token::LeftBrace);
                it.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                it.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                it.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
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
            '<' => tokens.push(get_comment(&mut it)?),
            '"' => tokens.push(get_string(&mut it)?),
            '0'..='9' | '+' | '-' => tokens.push(get_number(&mut it)?),
            'a'..='z' | 'A'..='Z' => tokens.push(get_identifier(&mut it)?),
            ' ' | '\t' | '\r' | '\n' => {
                it.next();
            }
            _ => {
                return Err(KvParseError::UnexpectedToken);
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
            Token::LeftBracket,
            Token::LeftBrace,
            Token::RightBracket,
            Token::RightBrace,
            Token::Comma,
            Token::Equal,
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn comment() -> Result<(), KvParseError> {
        let input = r#"<!-- This is a comment -->"#;
        let expected = [Token::Comment("This is a comment".to_string())];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn empty_comment() -> Result<(), KvParseError> {
        let input = r#"<!---->"#;
        let expected = [Token::Comment("".to_string())];

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
    fn identifier() -> Result<(), KvParseError> {
        let input = "name";
        let expected = [Token::Identifier("name".to_string())];

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
            Token::LeftBrace,
            Token::Identifier("name".to_string()),
            Token::Equal,
            Token::LeftBracket,
            Token::String("1".to_string()),
            Token::Comma,
            Token::Float(3.14159),
            Token::Comma,
            Token::LeftBrace,
            Token::RightBrace,
            Token::RightBracket,
            Token::RightBrace,
        ];

        let actual = tokenize(input)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
