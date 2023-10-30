use std::{fmt, fs, path};

use crate::{Position, Range, Symbol, SymbolTable, Token};

pub enum TextGenError {
    NoSuchFile(Range),
    FailedToReadFile(Range),
}

impl fmt::Debug for TextGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextGenError::NoSuchFile(range) => write!(f, "{:?} : Invalid file path", range),
            TextGenError::FailedToReadFile(range) => {
                write!(f, "{:?} : Failed to read file at path", range)
            }
        }
    }
}

pub fn to_output_tokens(
    symbols: &[Symbol],
    symbol_table: &SymbolTable,
) -> Result<Vec<Token>, TextGenError> {
    // TODO: think of how to do this using an iterator
    let mut tokens = vec![];
    let mut cursor_position = Position { line: 0, column: 0 };
    let mut last_token_end_position = cursor_position;
    for symbol in symbols {
        let token = to_token(
            symbol,
            symbol_table,
            &last_token_end_position,
            &cursor_position,
        )?;
        if let Token::Word { range, .. } = &token {
            cursor_position = range.end_pos;
            last_token_end_position = symbol.end_pos();
            tokens.push(token);
        } else {
            unreachable!("to_token must always return a word");
        }
    }
    Ok(tokens)
}

impl Symbol {
    fn end_pos(&self) -> Position {
        match self {
            Symbol::Word { range, .. } => range.end_pos,
            Symbol::Replace { range, .. } => range.end_pos,
            Symbol::Spread { range, .. } => range.end_pos,
        }
    }
}

fn to_token(
    symbol: &Symbol,
    symbol_table: &SymbolTable,
    last_token_end_position: &Position,
    cursor_position: &Position,
) -> Result<Token, TextGenError> {
    match symbol {
        Symbol::Word { text, range } => Ok(Token::Word {
            text: text.to_string(),
            range: calculate_new_range(last_token_end_position, cursor_position, range),
        }),
        Symbol::Replace { identifier, range } => {
            let text = symbol_table.get_variable(identifier).unwrap(); // We have already checked
            let range = calculate_replacement_range(cursor_position, &range.start_pos, &text);
            Ok(Token::Word { text, range })
        }
        Symbol::Spread { identifier, range } => {
            let text = get_file_content(symbol_table.get_variable(identifier).unwrap(), *range)?;
            let range = calculate_replacement_range(cursor_position, &range.start_pos, &text);
            Ok(Token::Word { text, range })
        }
    }
}

fn get_file_content(file_path: String, range: Range) -> Result<String, TextGenError> {
    let file_path = path::PathBuf::from(file_path);
    if !file_path.is_file() {
        return Err(TextGenError::NoSuchFile(range));
    }
    match fs::read_to_string(file_path) {
        Ok(text) => Ok(text.trim().to_string()),
        Err(_) => Err(TextGenError::FailedToReadFile(range)),
    }
}

fn calculate_replacement_range(
    cursor_position: &Position,
    replacement_start_pos: &Position,
    text: &String,
) -> Range {
    if cursor_position.line < replacement_start_pos.line {
        let start_pos = *replacement_start_pos;
        let end_pos = Position {
            line: replacement_start_pos.line,
            column: replacement_start_pos.column + text.len(),
        };
        Range { start_pos, end_pos }
    } else {
        let offset = replacement_start_pos.column - cursor_position.column;
        let column = cursor_position.column + offset;
        let line = std::cmp::max(replacement_start_pos.line, cursor_position.line);
        let start_pos = Position { line, column };
        let end_pos = Position {
            line,
            column: column + text.len(),
        };
        Range { start_pos, end_pos }
    }
}

fn calculate_new_range(
    last_token_end_position: &Position,
    cursor_position: &Position,
    current_range: &Range,
) -> Range {
    let Range { start_pos, end_pos } = current_range;
    if last_token_end_position == cursor_position || start_pos.line > cursor_position.line {
        return *current_range;
    }
    assert!(start_pos.line == cursor_position.line);
    let line = start_pos.line;
    let offset = start_pos.column - last_token_end_position.column;
    let length = end_pos.column - start_pos.column;
    let start_column = cursor_position.column + offset;
    Range {
        start_pos: Position {
            line,
            column: start_column,
        },
        end_pos: Position {
            line,
            column: start_column + length,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{back::calculate_new_range, Position, Range};

    #[test]
    fn test_calculate_new_range_return_same_for_new_line() {
        let expected = Range {
            start_pos: Position { line: 6, column: 0 },
            end_pos: Position { line: 6, column: 4 },
        };
        assert_eq!(
            calculate_new_range(
                &Position { line: 5, column: 6 },
                &Position { line: 5, column: 4 },
                &expected
            ),
            expected
        );
        let expected = Range {
            start_pos: Position {
                line: 10,
                column: 0,
            },
            end_pos: Position {
                line: 10,
                column: 4,
            },
        };
        assert_eq!(
            calculate_new_range(
                &Position { line: 5, column: 6 },
                &Position { line: 5, column: 4 },
                &expected
            ),
            expected
        );
    }

    #[test]
    fn test_calculate_new_range_return_same_for_no_change() {
        let expected = Range {
            start_pos: Position { line: 5, column: 6 },
            end_pos: Position { line: 5, column: 6 },
        };
        assert_eq!(
            calculate_new_range(
                &Position { line: 5, column: 6 },
                &Position { line: 5, column: 6 },
                &expected
            ),
            expected
        );
    }

    #[test]
    fn test_calculate_new_range_when_cursor_is_ahead() {
        let actual = calculate_new_range(
            &Position { line: 5, column: 6 },
            &Position {
                line: 5,
                column: 10,
            },
            &Range {
                start_pos: Position { line: 5, column: 6 },
                end_pos: Position { line: 5, column: 7 },
            },
        );
        assert_eq!(
            actual,
            Range {
                start_pos: Position {
                    line: 5,
                    column: 10
                },
                end_pos: Position {
                    line: 5,
                    column: 11
                }
            }
        )
    }

    #[test]
    fn test_calculate_new_range_when_cursor_is_behind() {
        let actual = calculate_new_range(
            &Position { line: 5, column: 6 },
            &Position { line: 5, column: 4 },
            &Range {
                start_pos: Position { line: 5, column: 6 },
                end_pos: Position { line: 5, column: 7 },
            },
        );
        assert_eq!(
            actual,
            Range {
                start_pos: Position { line: 5, column: 4 },
                end_pos: Position { line: 5, column: 5 }
            }
        )
    }
}
