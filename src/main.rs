use rlox::{interpreter::Interpreter, parser::Parser, scanner::Scanner};
use std::env;
use std::io::{self, Write};

fn main() {
    let mut interpreter = Interpreter::new();
    let args = env::args();
    let args: Vec<String> = args.collect();
    if let Some(file_path) = args.get(1) {
        let code = std::fs::read_to_string(file_path).expect("Cant read file");
        run(&code, &mut interpreter);
    } else {
        let mut s = String::new();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let read_status = io::stdin().read_line(&mut s);
            if let Err(_) = read_status {
                break;
            };
            run(&s, &mut interpreter);
            s.clear();
        }
    }
}

fn run(source: &String, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    // println!("{:#?}", scanner.tokens);
    let mut parser = Parser::new(scanner.tokens);
    let statments = parser.parse();
    if let Err(errors) = statments {
        for e in errors.iter() {
            println!(
                "[Error while parsing {} at line {}]: {}",
                e.error_type, e.line, e.message
            );
        }
        return;
    };
    // println!("{:#?}", statments);
    if let Err(e) = interpreter.interpret(statments.unwrap()) {
        println!("[RuntimeError]: {}", e);
    };
}
