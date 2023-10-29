use std::fmt;

pub enum ParseError {
    UnexpectedToken(Position),
    InvalidFilePath,
    FailedToOpenFile,
    VariableNotFound(Range),
    FileNotFound(Range),
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
            Self::VariableNotFound(pos) => {
                write!(f, "{:?} : variable not found", pos)
            }
            Self::FileNotFound(pos) => {
                write!(f, "{:?} : file not found", pos)
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Range {
    start_pos: Position,
    end_pos: Position,
}

impl fmt::Debug for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{:?}", self.start_pos, self.end_pos)
    }
}

impl From<(&Position, &Position)> for Range {
    fn from(value: (&Position, &Position)) -> Self {
        Range {
            start_pos: *value.0,
            end_pos: *value.1,
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

// TODO: introduce range
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Word { text: String, range: Range },
    Punctuation { value: char, pos: Position },
}

impl Token {
    fn start_pos(&self) -> Position {
        match self {
            Token::Word { range, .. } => range.start_pos,
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
                    range: Range {
                        start_pos: pos,
                        end_pos: Position { line, column: end },
                    },
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
                range: Range { start_pos, end_pos },
            })
        }
    }
}

pub type Identifier = String;

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Word { text: String },

    Replace { identifier: Identifier },

    Spread { identifier: Identifier },
}

pub struct SymbolTable {
    variables: std::collections::HashMap<Identifier, String>,
    files: std::collections::HashMap<Identifier, std::path::PathBuf>,
}

impl SymbolTable {
    pub fn new(variables: &[(&str, &str)], files: &[(&str, &std::path::Path)]) -> SymbolTable {
        let variables = variables
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let files = files
            .iter()
            .map(|(identifier, path)| (identifier.to_string(), std::path::PathBuf::from(path)))
            .collect();
        SymbolTable { variables, files }
    }

    fn has_variable(&self, identifier: &str) -> bool {
        self.variables.contains_key(identifier)
    }

    fn has_file(&self, identifier: &str) -> bool {
        self.files.contains_key(identifier)
    }
}

pub fn parse_tokens(tokens: &[Token], symbols: &SymbolTable) -> Result<Vec<Symbol>, ParseError> {
    match tokens {
        [] => Ok(vec![]),
        [Token::Word { text, .. }, rest @ ..] => Ok(vec![Symbol::Word {
            text: text.to_string(),
        }]
        .into_iter()
        .chain(parse_tokens(rest, symbols)?)
        .collect()),
        [Token::Punctuation {
            value: '$',
            pos: start_pos,
        }, Token::Punctuation { value: '{', .. }, Token::Word {
            text: identifier, ..
        }, Token::Punctuation {
            value: '}',
            pos: end_pos,
        }, rest @ ..] => {
            if symbols.has_variable(identifier) {
                Ok(vec![Symbol::Replace {
                    identifier: identifier.to_string(),
                }]
                .into_iter()
                .chain(parse_tokens(rest, symbols)?)
                .collect())
            } else {
                Err(ParseError::VariableNotFound((start_pos, end_pos).into()))
            }
        }
        [Token::Punctuation {
            value: '$',
            pos: start_pos,
        }, Token::Punctuation { value: '{', .. }, Token::Punctuation { value: '.', .. }, Token::Punctuation { value: '.', .. }, Token::Punctuation { value: '.', .. }, Token::Word {
            text: identifier, ..
        }, Token::Punctuation {
            value: '}',
            pos: end_pos,
        }, rest @ ..] => {
            if symbols.has_file(identifier) {
                // TODO: check if the identifier is valid
                Ok(vec![Symbol::Spread {
                    identifier: identifier.to_string(),
                }]
                .into_iter()
                .chain(parse_tokens(rest, symbols)?)
                .collect())
            } else {
                Err(ParseError::FileNotFound((start_pos, end_pos).into()))
            }
        }
        [Token::Punctuation { value, .. }, rest @ ..] => Ok(vec![Symbol::Word {
            text: value.to_string(),
        }]
        .into_iter()
        .chain(parse_tokens(rest, symbols)?)
        .collect()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::front::{create_tokens, reconstruct_text, ParseError, Symbol, SymbolTable};

    use super::{parse_tokens, Position, Range};

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

    #[test]
    fn test_parsing_just_text() {
        let symbols = parse_tokens(
            &create_tokens("Hello world!".to_string(), 0).unwrap(),
            &SymbolTable::new(&[], &[]),
        )
        .unwrap();
        assert_eq!(
            symbols,
            vec![
                Symbol::Word {
                    text: "Hello".to_string()
                },
                Symbol::Word {
                    text: "world".to_string()
                },
                Symbol::Word {
                    text: "!".to_string()
                }
            ]
        );
    }

    #[test]
    fn test_parsing_replace() {
        let symbols = parse_tokens(
            &create_tokens("Hello ${var1}! ${var2}".to_string(), 0).unwrap(),
            &SymbolTable::new(&[("var1", ""), ("var2", "")], &[]),
        )
        .unwrap();
        assert_eq!(
            symbols,
            vec![
                Symbol::Word {
                    text: "Hello".to_string()
                },
                Symbol::Replace {
                    identifier: "var1".to_string()
                },
                Symbol::Word {
                    text: "!".to_string()
                },
                Symbol::Replace {
                    identifier: "var2".to_string()
                }
            ]
        );
    }

    #[test]
    fn test_parsing_replace_err() {
        let symbols = parse_tokens(
            &create_tokens("Hello ${var1}! ${var2}".to_string(), 0).unwrap(),
            &SymbolTable::new(&[], &[]),
        );
        let err_pos: Range = (
            &Position { line: 0, column: 6 },
            &Position {
                line: 0,
                column: 12,
            },
        )
            .into();
        if let Err(ParseError::VariableNotFound(r)) = symbols {
            assert_eq!(r, err_pos)
        } else {
            panic!("Expected an error");
        }
    }

    #[test]
    fn test_parsing_spread_err() {
        let symbols = parse_tokens(
            &create_tokens("Hello ${...var1}! ${var2}".to_string(), 0).unwrap(),
            &SymbolTable::new(&[], &[]),
        );
        let err_pos: Range = (
            &Position { line: 0, column: 6 },
            &Position {
                line: 0,
                column: 15,
            },
        )
            .into();
        if let Err(ParseError::FileNotFound(r)) = symbols {
            assert_eq!(r, err_pos);
        } else {
            panic!("Expected an error");
        }
    }

    #[test]
    fn test_parsing_spread() {
        let symbols = parse_tokens(
            &create_tokens("Hello ${...var1}! ${...var2}".to_string(), 0).unwrap(),
            &SymbolTable::new(
                &[],
                &[
                    ("var1", PathBuf::from("").as_path()),
                    ("var2", PathBuf::from("").as_path()),
                ],
            ),
        )
        .unwrap();
        assert_eq!(
            symbols,
            vec![
                Symbol::Word {
                    text: "Hello".to_string()
                },
                Symbol::Spread {
                    identifier: "var1".to_string()
                },
                Symbol::Word {
                    text: "!".to_string()
                },
                Symbol::Spread {
                    identifier: "var2".to_string()
                }
            ]
        );
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
            range: Range { start_pos, end_pos },
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
