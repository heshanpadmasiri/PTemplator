use std::fmt;
enum ParseError {
    UnexpectedToken(Position),
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(pos) => {
                write!(f, "{:?} : Unexpected token", pos)
            }
        }
    }
}

// TODO: also include the file path
// NOTE: positions are starting from 0
#[derive(PartialEq, Eq)]
pub struct Position {
    line: usize,
    column: usize,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: also show the file name in error
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

fn create_tokens(text: String, line: usize) -> Result<Vec<Token>, ParseError> {
    let mut char_buffer = vec![];
    let mut tokens = vec![];
    let mut start = 0;
    for (i, char) in text.chars().enumerate() {
        match char {
            ' ' => {
                tokens.push(create_token(&char_buffer, line, start, i));
                char_buffer.clear();
                start = i + 1;
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
    Ok(tokens.into_iter().filter_map(|opt| opt).collect())
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

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_tokenize_simple_line() {
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

    fn create_word(t: &str, start: usize) -> crate::Token {
        let text = t.to_string();
        let start_pos = crate::Position {
            line: 0,
            column: start,
        };
        let end_pos = crate::Position {
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
        let pos = crate::Position {
            column: start,
            line: 0,
        };
        crate::Token::Punctuation { value, pos }
    }
}
