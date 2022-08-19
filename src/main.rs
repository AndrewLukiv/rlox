use rlox::parser::ParsingErrorType;
use rlox::{interpreter::Interpreter, parser::Parser, scanner::Scanner};
use std::env;
use std::io::{self, Write};

fn main() {
    let mut interpreter = Interpreter::new();
    let args = env::args();
    let args: Vec<String> = args.collect();
    if let Some(file_path) = args.get(1) {
        let code = std::fs::read_to_string(file_path).expect("Cant read file");
        run(&code, &mut interpreter, false);
    } else {
        let mut s = String::new();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let read_status = io::stdin().read_line(&mut s);
            if let Err(_) = read_status {
                break;
            };
            run(&s, &mut interpreter, true);
            s.clear();
        }
    }
}

fn run(source: &String, interpreter: &mut Interpreter, repl_mode: bool) {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    // println!("{:#?}", scanner.tokens);
    let mut parser = Parser::new(scanner.tokens);
    let statments = parser.parse();
    if let Err(errors) = statments {
        if repl_mode
            && errors.len() == 1
            && errors[0].error_type == ParsingErrorType::Stmt
            && errors[0].expression.is_some()
        {
            let expr = errors[0].expression.as_ref().unwrap();
            match interpreter.evaluate(expr) {
                Ok(value) => {
                    println!("{value:?}");
                }
                Err(e) => eprintln!("{e}"),
            };
            return;
        }
        for e in errors.iter() {
            eprintln!(
                "[Error while parsing {} at line {}]: {}",
                e.error_type, e.line, e.message
            );
        }
        return;
    };
    // println!("{:#?}", statments);
    if let Err(e) = interpreter.interpret(statments.unwrap()) {
        eprintln!("[RuntimeError]: {}", e);
    };
}
