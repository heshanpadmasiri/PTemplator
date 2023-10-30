use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path;

mod back;
mod front;
use front::{create_tokens, parse_tokens, reconstruct_text, ParseError};

use crate::back::to_output_tokens;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Range {
    start_pos: Position,
    end_pos: Position,
}

// NOTE: positions are starting from 0
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Position {
    line: usize,
    column: usize,
}

type Identifier = String;

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Word {
        text: String,
        range: Range,
    },

    Replace {
        identifier: Identifier,
        range: Range,
    },

    Spread {
        identifier: Identifier,
        range: Range,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Word { text: String, range: Range },
    Punctuation { value: char, pos: Position },
}

pub struct SymbolTable {
    variables: std::collections::HashMap<Identifier, String>,
}

impl SymbolTable {
    pub fn new<S: AsRef<str>>(variables: &[(S, S)]) -> SymbolTable {
        let variables = variables
            .iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect();
        SymbolTable { variables }
    }

    fn has_variable(&self, identifier: &str) -> bool {
        self.variables.contains_key(identifier)
    }

    pub fn get_variable(&self, identifier: &str) -> Option<String> {
        self.variables.get(identifier).cloned()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = path::PathBuf::from(args[1].clone());
    let symbol_table = symbol_table_from_args(&args[2..]);
    match tokenize_file(&file_path) {
        Err(err) => print_error(err, &file_path),
        Ok(tokens) => match parse_tokens(&tokens, &symbol_table) {
            Err(err) => print_error(err, &file_path),
            Ok(symbols) => {
                let output_tokens = to_output_tokens(&symbols, &symbol_table);
                println!("{}", reconstruct_text(&output_tokens.unwrap()));
            }
        },
    }
}

fn symbol_table_from_args(args: &[String]) -> SymbolTable {
    SymbolTable::new(
        &args
            .chunks(2)
            .map(|chunk| match chunk {
                [key, value] => (parse_identifier(key), parse_variable(value)),
                _ => {
                    panic!("Invalid symbol {:?}", chunk)
                }
            })
            .collect::<Vec<(String, String)>>(),
    )
}

fn parse_identifier(value: &str) -> Identifier {
    if !value.starts_with("--") {
        panic!(
            "Invalid variable name {}: use --<VarName> <Var value>",
            value
        )
    }
    value[2..].to_string()
}

fn parse_variable(value: &str) -> String {
    if value.starts_with('"') {
        if !value.ends_with('"') {
            panic!("Varible value not terminated {}", value)
        }
        value[1..(value.len() - 1)].to_string()
    } else {
        value.to_string()
    }
}

fn print_error(err: ParseError, file_path: &path::Path) {
    eprintln!("{}:{:?}", file_path.to_str().unwrap(), err);
}

fn tokenize_file(file_path: &path::Path) -> Result<Vec<Token>, ParseError> {
    if !file_path.is_file() {
        return Err(ParseError::InvalidFilePath);
    }
    match File::open(file_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            reader
                .lines()
                .enumerate()
                .try_fold(Vec::new(), |acc, (i, l)| match l {
                    Ok(t) => Ok(acc.into_iter().chain(create_tokens(t, i)?).collect()),
                    Err(_) => Err(ParseError::FailedToReadLine(i)),
                })
        }
        Err(_) => Err(ParseError::FailedToOpenFile),
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufRead, io::BufReader, path};

    use crate::{
        front::{create_tokens, parse_tokens, reconstruct_text},
        symbol_table_from_args, to_output_tokens, SymbolTable,
    };

    #[test]
    fn test_roundtrip_simple_file() {
        let file_path = path::PathBuf::from("./test_corpus/simple.txt");
        let tokens = crate::tokenize_file(&file_path).unwrap();
        let expected_text = crate::front::reconstruct_text(&tokens);
        let actual_text = read_file_as_string(&file_path);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn test_arg_parsing() {
        let args: Vec<String> = ["--var1", "1", "--var2", "\"2\""]
            .iter()
            .map(|each| each.to_string())
            .collect();
        let symbols = symbol_table_from_args(&args);
        assert_eq!(symbols.get_variable("var2").unwrap(), "2".to_string());
    }

    #[test]
    fn test_roundtrip_spread_simple() {
        let file_path = path::PathBuf::from("./test_corpus/spread.txt");
        let symbol_table = SymbolTable::new::<&str>(&[("file1", "./test_corpus/spread_content.txt")]);
        let tokens = create_tokens(read_file_as_string(&file_path), 0).unwrap();
        let symbols = parse_tokens(&tokens, &symbol_table).unwrap();
        let output_tokens = to_output_tokens(&symbols, &symbol_table);
        let output = reconstruct_text(&output_tokens.unwrap());
        assert_eq!(output, "bb aa Foo Bar\nBaz cc".to_string())
    }

    #[test]
    fn test_roundtrip_replace_simple() {
        let symbol_table = SymbolTable::new::<&str>(&[("var1", "world")]);
        let tokens = create_tokens("Hello ${var1}!".to_string(), 0).unwrap();
        let symbols = parse_tokens(&tokens, &symbol_table).unwrap();
        let output_tokens = to_output_tokens(&symbols, &symbol_table);
        let output = reconstruct_text(&output_tokens.unwrap());
        assert_eq!(output, "Hello world!".to_string())
    }

    fn read_file_as_string(path: &path::Path) -> String {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        reader
            .lines()
            .map(|l| l.unwrap())
            .collect::<Vec<String>>()
            .join("\n")
    }
}
