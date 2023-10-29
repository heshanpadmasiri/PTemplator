use std::fmt;

pub enum ParseError {
    UnexpectedToken(Position),
    InvalidFilePath,
    FailedToOpenFile,
    FailedToReadLine(usize),
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(pos) => {
                write!(f, "{:?} : Unexpected token", pos)
            }
            Self::InvalidFilePath => {
                write!(f, ": Invalid file path")
            }
            Self::FailedToOpenFile => {
                write!(f, ": Failed to open file")
            }
            Self::FailedToReadLine(l) => {
                write!(f, "{}: Failed to read line", l)
            }
        }
    }
}

// NOTE: positions are starting from 0
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Position {
    line: usize,
    column: usize,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Word {
        text: String,
        start_pos: Position,
        end_pos: Position, // NOTE: end is exclusive
    },
    Punctuation {
        value: char,
        pos: Position,
    },
}

impl Token {
    fn start_pos(&self) -> Position {
        match self {
            Token::Word { start_pos, .. } => *start_pos,
            Token::Punctuation { pos, .. } => *pos,
        }
    }
}

impl std::string::ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Word { text, .. } => text.to_string(),
            Token::Punctuation { value, .. } => value.to_string(),
        }
    }
}

pub fn reconstruct_text(tokens: &[Token]) -> String {
    let mut current_line = 0;
    let mut current_column = 0;
    let mut parts: Vec<String> = vec![];
    for token in tokens {
        let Position { line, column } = token.start_pos();
        while current_line < line {
            parts.push("\n".to_string());
            current_line += 1;
            current_column = 0;
        }
        while current_column < column {
            parts.push(" ".to_string());
            current_column += 1;
        }
        let part = token.to_string();
        current_column += part.len();
        parts.push(part);
    }
    parts.concat()
}

pub fn create_tokens(text: String, line: usize) -> Result<Vec<Token>, ParseError> {
    let mut char_buffer = vec![];
    let mut tokens = vec![];
    let mut start = 0;
    // TODO: turn in to iterator
    for (i, char) in text.chars().enumerate() {
        match char {
            ' ' => {
                tokens.push(create_token(&char_buffer, line, start, i));
                char_buffer.clear();
                start = i + 1;
            }
            c if c.is_whitespace() => {
                return Err(ParseError::UnexpectedToken(Position { line, column: i }));
            }
            c if c.is_ascii_punctuation() => {
                // Add chars upto this
                // TODO: common code refactor
                tokens.push(create_token(&char_buffer, line, start, i));
                char_buffer.clear();

                char_buffer.push(char);
                tokens.push(create_token(&char_buffer, line, i, i + 1));
                char_buffer.clear();
                start = i + 1;
            }
            _ => {
                char_buffer.push(char);
            }
        }
    }
    Ok(tokens.into_iter().flatten().collect())
}

fn create_token(chars: &[char], line: usize, start: usize, end: usize) -> Option<Token> {
    match chars.len() {
        0 => None,
        1 => {
            assert_eq!(start + 1, end);
            let char = chars[0];
            let pos = Position {
                line,
                column: start,
            };
            if char.is_ascii_punctuation() {
                Some(Token::Punctuation { value: char, pos })
            } else {
                Some(Token::Word {
                    text: char.to_string(),
                    start_pos: pos,
                    end_pos: Position { line, column: end },
                })
            }
        }
        n => {
            assert_eq!(start + n, end);
            let start_pos = Position {
                line,
                column: start,
            };
            let end_pos = Position { line, column: end };
            let text = chars.iter().collect();
            Some(Token::Word {
                text,
                start_pos,
                end_pos,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::front::{create_tokens, reconstruct_text};

    use super::Position;

    #[test]
    fn test_tokenize_simple_line() {
        let l = "Hello world!".to_string();
        let tokens = create_tokens(l, 0).unwrap();
        let expected = vec![
            create_word("Hello", 0),
            create_word("world", "Hello".len() + 1),
            create_punctuation("!", "Hello world".len()),
        ];
        println!("{:?}", tokens);
        assert_eq!(expected.len(), tokens.len());
        for i in 0..expected.len() {
            assert_eq!(tokens[i], expected[i]);
        }
    }

    #[test]
    fn test_roundtrip_simple_line() {
        let l = "Hello world!".to_string();
        let tokens = create_tokens(l.clone(), 0).unwrap();
        let r = reconstruct_text(&tokens);
        assert_eq!(r, l);
    }

    fn create_word(t: &str, start: usize) -> crate::Token {
        let text = t.to_string();
        let start_pos = Position {
            line: 0,
            column: start,
        };
        let end_pos = Position {
            line: 0,
            column: start + text.len(),
        };
        crate::Token::Word {
            text,
            start_pos,
            end_pos,
        }
    }

    fn create_punctuation(t: &str, start: usize) -> crate::Token {
        let chars = t.chars().take(1).collect::<Vec<char>>();
        let value = chars[0];
        let pos = Position {
            column: start,
            line: 0,
        };
        crate::Token::Punctuation { value, pos }
    }
}
