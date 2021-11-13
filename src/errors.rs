use std::fmt;
use std::process;

use crate::source_file;

pub struct ErrorDescription {
	pub subject: String,
	pub location: source_file::SourceSpan,
	pub description: String,
}

impl fmt::Display for ErrorDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let ErrorDescription {
			subject,
			location,
			description,
		} = self;
		write!(
			f,
			"[line: {}, col: {}] Error ({}): {}",
			location.start.line, location.start.column, description, subject
		)
	}
}

pub enum Error {
	Syntax(ErrorDescription),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Error::Syntax(description) = self;
		write!(f, "{}", description)
	}
}

pub struct ErrorLog {
	pub errors: Vec<Error>,
}

impl ErrorLog {
	pub fn new() -> Self {
		ErrorLog { errors: Vec::new() }
	}
	pub fn log(
		&mut self,
		location: source_file::SourceSpan,
		subject: &str,
		description: &str,
	) -> &Self {
		self.errors.push(Error::Syntax(ErrorDescription {
			subject: String::from(subject),
			location,
			description: String::from(description),
		}));
		self
	}
	pub fn len(&self) -> usize {
		self.errors.len()
	}
}

impl fmt::Display for ErrorLog {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut result = String::new();
		for error in self.errors.iter() {
			result.push_str(&error.to_string());
		}
		write!(f, "{}", result)
	}
}

pub fn exit_with_code(code: exitcode::ExitCode) {
	process::exit(code);
}

pub fn exit_on_error(code: exitcode::ExitCode, error_log: &ErrorLog) {
	println!("{}", error_log);
	exit_with_code(code);
}
