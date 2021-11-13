// TODO: Make a struct that actually contains the source.

// -----| Locations |-----

/// A SourceLocation represents a single symbol and where it's location in source.
#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
	pub line: usize,
	pub column: usize,
	/// The absolute index into the source, regardless of which line or or column.
	pub index: usize,
}

impl SourceLocation {
	pub fn new() -> Self {
		SourceLocation {
			line: 1,
			column: 1,
			index: 0,
		}
	}
	pub fn increment_line(&mut self) {
		self.line += 1;
		self.column = 1;
		self.index += 1;
	}
	pub fn increment_column(&mut self) {
		self.column += 1;
		self.index += 1;
	}
	pub fn increment(&mut self, symbol: &str) {
		if symbol == "\n" {
			self.increment_line();
		} else {
			self.increment_column();
		}
	}
}

/// SourceLocations represent one to many symbols in linear sequence in source.
#[derive(Debug, Clone, Copy)]
pub struct SourceSpan {
	/// Inclusive/Open
	pub start: SourceLocation,
	/// Exclusive/Closed
	pub end: SourceLocation,
}

impl SourceSpan {
	pub fn new() -> Self {
		SourceSpan {
			start: SourceLocation::new(),
			end: SourceLocation::new(),
		}
	}
	pub fn close(&mut self) {
		self.start = self.end;
	}
}
