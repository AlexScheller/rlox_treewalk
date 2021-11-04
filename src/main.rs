use exitcode;
use std::env;
use std::fs;
use std::process;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() > 2 {
		println!("Usage: rlox <script>");
		process::exit(exitcode::USAGE);
	} else if args.len() == 2 {
		run_file(&args[1]);
	} else {
		run_prompt();
	}
}

fn run_file(file_name: &String) {
	let contents = fs::read_to_string(file_name).expect("Failed to read file");
	println!("{}", contents);
}

fn run_prompt() {
	println!("Prompt!");
}
