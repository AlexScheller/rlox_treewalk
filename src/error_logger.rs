use std::fmt;

pub struct ErrorDescription {
	pub target: String,
	pub line: usize,
	pub column: usize,
	pub description: String,
}

impl fmt::Display for ErrorDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let ErrorDescription {
			target,
			line,
			column,
			description,
		} = self;
		write!(
			f,
			"[line: {}, col: {}] Error ({}): {}",
			line, column, target, description
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
	pub fn log(&mut self, line: usize, column: usize, target: &str, description: &str) -> &Self {
		self.errors.push(Error::Syntax(ErrorDescription {
			target: String::from(target),
			line,
			column,
			description: String::from(description),
		}));
		self
	}
	pub fn len(&self) -> usize {
		self.errors.len()
	}
}
