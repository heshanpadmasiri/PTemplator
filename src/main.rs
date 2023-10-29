use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path;

mod front;
use front::{reconstruct_text, ParseError, Token, create_tokens};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = path::PathBuf::from(args[1].clone());
    let tokens = tokenize_file(&file_path);
    match tokens {
        Err(err) => print_error(err, &file_path),
        Ok(tokens) => println!("{:?} \n {}", tokens, reconstruct_text(&tokens)),
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

    #[test]
    fn test_roundtrip_simple_file() {
        let file_path = path::PathBuf::from("./test_corpus/simple.txt");
        let tokens = crate::tokenize_file(&file_path).unwrap();
        let expected_text = crate::front::reconstruct_text(&tokens);
        let actual_text = read_file_as_string(&file_path);
        assert_eq!(expected_text, actual_text);
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
