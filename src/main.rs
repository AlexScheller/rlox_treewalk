use exitcode;
use std::env;
use std::fs;
use std::io;
use std::io::Write;

mod errors;
mod language_utilities;
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
	for token in scanner.tokens() {
		println!("{:?}", token);
	}
}
