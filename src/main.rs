use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path;

mod front;
use front::{create_tokens, parse_tokens, reconstruct_text, Identifier, ParseError, Token};

use crate::front::SymbolTable;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = path::PathBuf::from(args[1].clone());
    let symbol_table = symbol_table_from_args(&args[2..]);
    match tokenize_file(&file_path) {
        Err(err) => print_error(err, &file_path),
        Ok(tokens) => {
            println!("{:?} \n {}", &tokens, reconstruct_text(&tokens));
            match parse_tokens(&tokens, &symbol_table) {
                Err(err) => print_error(err, &file_path),
                Ok(symbols) => {
                    println!("{:?}", &symbols);
                }
            }
        }
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

    use crate::symbol_table_from_args;

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
