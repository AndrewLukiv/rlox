use rlox::scanner::Scanner;
use std::env;

fn main() {
    let args = env::args();
    let args: Vec<String> = args.collect();
    if let Some(file_path) = args.get(1) {
        let code = std::fs::read_to_string(file_path).expect("Cant read file");
        run(code);
    } else {
        panic!("You need provide path to file");
    }
}

fn run(source: String) {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    for token in scanner.tokens {
        println!("{:#?}", token)
    }
}
