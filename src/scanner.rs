#[derive(Debug, Clone)]
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
	// Terminal
	Eof,
}

#[derive(Debug, Clone)]
pub struct SourceToken {
	token: Token,
	line: usize,
	column: usize,
}

pub struct Scanner {
	source: String,
	tokens: Vec<SourceToken>,
}

impl Scanner {
	// --- Constructors ---
	pub fn new() -> Self {
		Scanner {
			source: String::new(),
			tokens: Vec::new(),
		}
	}
	pub fn from_source(source: &str) -> Self {
		let mut ret = Scanner::new();
		ret.tokenize(source);
		ret
	}
	// --- Accessors ---
	pub fn tokens(&self) -> Vec<SourceToken> {
		self.tokens.clone()
	}
	// --- Mutators ---
	pub fn tokenize(&mut self, source: &str) {
		self.source = String::from(source);
		self.tokens = Vec::new();
		let mut index = 0;
		while index < self.source.len() {
			index += 1;
		}
		self.tokens.push(SourceToken {
			token: Token::Eof,
			line: index,
			column: index,
		});
	}
}
