use exitcode;
use std::env;
use std::fs;
use std::io;
use std::io::Write;

mod ast_printer;
mod errors;
mod language_utilities;
mod parser;
mod scanner;
mod source_file;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox <script>");
        errors::exit_with_code(exitcode::USAGE);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
    // let expression = parser::Expr::Binary(parser::BinaryExpr {
    // 	left: Box::new(parser::Expr::Unary(parser::UnaryExpr {
    // 		operator: scanner::Token::Minus,
    // 		right: Box::new(parser::Expr::Literal(parser::LiteralKind::Number(123.0))),
    // 	})),
    // 	operator: scanner::Token::Star,
    // 	right: Box::new(parser::Expr::Grouping(Box::new(parser::Expr::Literal(
    // 		parser::LiteralKind::Number(45.67),
    // 	)))),
    // });
    // println!("{}", ast_printer::expr_to_ast_string(expression));
}

fn run_file(file_name: &str) {
    let contents = fs::read_to_string(file_name).expect("Failed to read file");
    run(contents);
}

fn print_flush(str: &str) {
    print!("{}", str);
    io::stdout().flush().expect("Failed to flush output");
}

fn run_prompt() {
    loop {
        let mut line = String::new();
        print_flush("> ");
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read user input");
        if line == "\n" {
            break;
        }
        run(line);
    }
}

fn run(source: String) {
    let scanner = scanner::Scanner::from_source(source);
    let mut parser = parser::Parser::new(scanner.tokens());
    let expression = parser.parse();
    println!("{}", ast_printer::expr_to_ast_string(expression));
    // for token in scanner.tokens() {
    // 	println!("{:?}", token);
    // }
}
