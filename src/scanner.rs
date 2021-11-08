#[derive(Debug)]
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

#[derive(Debug)]
pub struct SourceToken {
	token: Token,
	line: usize,
	column: usize,
}

pub struct SourceTokens {
	pub tokens: Vec<SourceToken>,
}

pub fn scan_tokens(source: String) -> SourceTokens {
	let mut tokens: Vec<SourceToken> = Vec::new();
	let mut index = 0;
	while index < source.len() {
		index += 1;
	}
	tokens.push(SourceToken {
		token: Token::Eof,
		line: index,
		column: index,
	});
	SourceTokens { tokens }
}
