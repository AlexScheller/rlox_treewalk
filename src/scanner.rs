use unicode_segmentation::UnicodeSegmentation;

use crate::errors;
use crate::source_file;

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
	// Meta
	// TODO: Break this up into newlines, tabs, spaces, etc.?
	Comment(String),
	Whitespace,
	Eof,
	Error, // This seems bad...
}

#[derive(Debug, Clone)]
pub struct SourceToken {
	token: Token,
	location_span: source_file::SourceSpan,
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
	cursor: source_file::SourceSpan,
	error_log: errors::ErrorLog,
}

impl Scanner {
	// --- Constructors ---
	pub fn new() -> Self {
		Scanner {
			source: Vec::new(), // TODO: Use a struct created in `source_file.rs`
			tokens: Vec::new(),
			cursor: source_file::SourceSpan::new(),
			error_log: errors::ErrorLog::new(),
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
				"/" => {
					// Comment
					if self.match_next_symbol("/") {
						let mut content = String::from("//");
						while let Some(symbol) = self.peek_next_symbol() {
							if symbol == "\n" {
								break;
							}
							content.push_str(&symbol);
							self.consume_next_symbol();
						}
						Token::Comment(content)
					} else {
						Token::Slash
					}
				}
				" " => Token::Whitespace,
				"\r" => Token::Whitespace,
				"\t" => Token::Whitespace,
				"\n" => Token::Whitespace,
				"\"" => {
					if let Some(string_value) = self.consume_string() {
						Token::String(string_value)
					} else {
						Token::Error
					}
				}
				_ => Token::Nil, // TODO: Get this working
			};
			let location_span = self.cursor;
			self.cursor.close();
			if token != Token::Error {
				Some(SourceToken {
					token,
					location_span,
				})
			} else {
				errors::exit_on_error(exitcode::DATAERR, &self.error_log);
				None
			}
		} else {
			None
		}
	}
	fn consume_next_symbol(&mut self) -> Option<Symbol> {
		if let Some(ret) = self.source.get(self.cursor.end.index) {
			self.cursor.end.increment(ret);
			Some(ret.to_string())
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
	fn peek_next_symbol(&self) -> Option<Symbol> {
		if let Some(curr) = self.source.get(self.cursor.end.index) {
			Some(curr.to_string())
		} else {
			None
		}
	}
	fn consume_string(&mut self) -> Option<String> {
		while let Some(symbol) = self.peek_next_symbol() {
			self.cursor.end.increment(&symbol);
			if symbol == "\"" {
				return Some(self.source_substring(self.cursor));
			}
		}
		let error_string = &self.source_substring(self.cursor);
		self.error_log
			.log(self.cursor, error_string, "Unterminated String");
		None
	}
	fn source_substring(&self, cursor: source_file::SourceSpan) -> String {
		self.source[cursor.start.index..cursor.end.index].join("")
	}
}
