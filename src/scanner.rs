use unicode_segmentation::UnicodeSegmentation;

use crate::error_logger;

const USE_EXTENDED_UNICODE: bool = true;

// -----| Symbols |-----

type Symbol = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
	// Single-character tokens
	LeftParen,
	RightParen,
	LeftBrace,
	RightBrace,
	Comma,
	Dot,
	Minus,
	Plus,
	Semicolon,
	Slash,
	Star,
	// One or two character tokens
	Bang,
	BangEqual,
	Equal,
	EqualEqual,
	Greater,
	GreaterEqual,
	Less,
	LessEqual,
	// Literals
	Identifier(String),
	String(String),
	Number(String),
	// Keywords
	And,
	Class,
	Else,
	False,
	Fun,
	For,
	If,
	Nil,
	Or,
	Print,
	Return,
	Super,
	This,
	True,
	Var,
	While,
	// File Sentries
	// TODO: Break this up into newlines, tabs, spaces, etc.?
	Whitespace,
	Eof,
}

#[derive(Debug, Clone)]
pub struct SourceToken {
	token: Token,
	location_span: SourceSpan,
}

// -----| Locations |-----

/// A SourceLocation represents a single symbol and where it's location in source.
#[derive(Debug, Clone, Copy)]
struct SourceLocation {
	line: usize,
	column: usize,
	/// The absolute index into the source, regardless of which line or or column.
	index: usize,
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
struct SourceSpan {
	/// Inclusive/Open
	start: SourceLocation,
	/// Exclusive/Closed
	end: SourceLocation,
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

// -----| Utilities |-----
// fn is_whitespace(symbol: &str) -> bool {
// 	match symbol {
// 		" " => true,
// 		"\t" => true,
// 		"\n" => true,
// 		_ => false,
// 	}
// }

/// The main object through which the source is consumed and transformed into a token sequence.
pub struct Scanner {
	/// UTF8 Graphemes
	source: Vec<String>,
	tokens: Vec<SourceToken>,
	/// The subset of the source currently being investigated
	cursor: SourceSpan,
	error_log: error_logger::ErrorLog,
}

impl Scanner {
	// --- Constructors ---
	pub fn new() -> Self {
		Scanner {
			source: Vec::new(),
			tokens: Vec::new(),
			cursor: SourceSpan::new(),
			error_log: error_logger::ErrorLog::new(),
		}
	}
	pub fn from_source(source: String) -> Self {
		let mut ret = Scanner::new();
		ret.tokenize(source);
		ret
	}
	// --- Accessors ---
	pub fn tokens(&self) -> Vec<SourceToken> {
		self.tokens.clone()
	}
	// --- Responsibilities ---
	fn tokenize(&mut self, raw_source: String) {
		self.source = raw_source
			.graphemes(USE_EXTENDED_UNICODE)
			.map(|grapheme| String::from(grapheme))
			.collect();
		while let Some(token) = self.scan_next_token() {
			self.tokens.push(token);
		}
	}
	fn scan_next_token(&mut self) -> Option<SourceToken> {
		if let Some(symbol) = self.consume_next_symbol() {
			let token = match symbol.as_ref() {
				"(" => Token::LeftParen,
				")" => Token::RightParen,
				"{" => Token::LeftBrace,
				"}" => Token::RightBrace,
				"," => Token::Comma,
				"." => Token::Dot,
				"-" => Token::Minus,
				"+" => Token::Plus,
				";" => Token::Semicolon,
				"*" => Token::Star,
				"!" => {
					if self.match_next_symbol("=") {
						Token::BangEqual
					} else {
						Token::Bang
					}
				}
				"=" => {
					if self.match_next_symbol("=") {
						Token::EqualEqual
					} else {
						Token::Equal
					}
				}
				"<" => {
					if self.match_next_symbol("=") {
						Token::LessEqual
					} else {
						Token::Less
					}
				}
				">" => {
					if self.match_next_symbol("=") {
						Token::GreaterEqual
					} else {
						Token::Greater
					}
				}
				// "/" => {
				// 	if self.match_next_symbol("/") {
				// 		// Comment, ignore till end of the line.
				// 	} else {
				// 		Token::Slash
				// 	}
				// }
				" " => Token::Whitespace,
				"\r" => Token::Whitespace,
				"\t" => Token::Whitespace,
				"\n" => Token::Whitespace,
				_ => Token::Nil, // TODO: Get this working
			};
			let location_span = self.cursor;
			self.cursor.close();
			Some(SourceToken {
				token,
				location_span,
			})
		} else {
			None
		}
	}
	fn consume_next_symbol(&mut self) -> Option<Symbol> {
		if let Some(ret) = self.source.get(self.cursor.end.index) {
			self.cursor.end.increment(ret);
			Some(String::from(ret))
		} else {
			None
		}
	}
	fn match_next_symbol(&mut self, target: &str) -> bool {
		if let Some(curr) = self.source.get(self.cursor.end.index) {
			if curr == target {
				// Technically we know that curr can never be a newline...
				self.cursor.end.increment(curr);
				return true;
			}
		};
		false
	}
}

// pub struct Scanner {
// 	source: String,
// 	source_graphemes: Vec<&str>,
// 	tokens: Vec<SourceToken>,
// 	cursor: SourceLocation,
// 	error_log: error_logger::ErrorLog,
// }

// // TODO, re-write this to remove some of the warts.
// impl Scanner {
// 	// --- Constructors ---
// 	pub fn new() -> Self {
// 		Scanner {
// 			source: String::new(),
// 			tokens: Vec::new(),
// 			cursor: SourceLocation::new(),
// 			error_log: error_logger::ErrorLog::new(),
// 		}
// 	}
// 	pub fn from_source(source: &str) -> Self {
// 		let mut ret = Scanner::new();
// 		ret.tokenize_source(source);
// 		ret
// 	}
// 	// --- Accessors ---
// 	pub fn tokens(&self) -> Vec<SourceToken> {
// 		self.tokens.clone()
// 	}
// 	fn is_source_consumed(&self) -> bool {
// 		self.cursor.index >= self
// 	}
// 	// --- Mutators ---
// 	fn tokenize_source(&mut self, source: &str) {
// 		// self.source = String::from(source);
// 		let symbols: Vec<&str> = self.source.graphemes(USE_EXTENDED_UNICODE).collect();
// 		while let Some(token) = self.scan_next_token(symbols) {
// 			self.tokens.push(token)
// 		}
// 	}
// 	fn scan_next_token(&mut self, symbols: Vec<&str>) -> Option<SourceToken> {
// 		let symbol = self.consume_next_symbol(symbols);
// 		let scan = match symbol {
// 			// Singles
// 			"(" => Some(Token::LeftParen),
// 			")" => Some(Token::RightParen),
// 			"{" => Some(Token::LeftBrace),
// 			"}" => Some(Token::RightBrace),
// 			"," => Some(Token::Comma),
// 			"." => Some(Token::Dot),
// 			"-" => Some(Token::Minus),
// 			"+" => Some(Token::Plus),
// 			";" => Some(Token::Semicolon),
// 			"*" => Some(Token::Star),
// 			// Pairs
// 			"!" => {
// 				if self.match_next_symbol("=", symbols) {
// 					Some(Token::BangEqual)
// 				} else {
// 					Some(Token::Bang)
// 				}
// 			}
// 			_ => {
// 				let description = format!("Unexpected Character: {}", symbol);
// 				// error_log.log(cursor.line, cursor.column, "", &description);
// 				None
// 			}
// 		};
// 	}
// 	fn consume_next_symbol(&self, symbols: Vec<&str>) -> &str {
// 		let ret = symbols[self.cursor.index];
// 		self.cursor.index += 1;
// 		ret
// 	}
// 	fn match_next_symbol(&self, target: &str, symbols: Vec<&str>) -> bool {
// 		if self.is_source_consumed() {
// 			false
// 		} else if symbols[self.cursor.index] != target {
// 			false
// 		} else {
// 			true
// 		}
// 	}
// }
