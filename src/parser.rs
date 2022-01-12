use crate::scanner;

// -----| Expression Grammer |-----
//
// In order of increasing precedence
//
// expression 	-> equality ;
// equality 	-> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison	-> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term			-> factor ( ( "-" | "+" ) factor )* ;
// factor		-> unary ( ( "/" | "*" ) unary )* ;
// unary		-> ( "!" | "-" ) unary | primary ;
// primary		-> NUMBER| | STRING | "true" | "false" | "nil" | "(" expression ")" ;

pub enum LiteralKind {
	Number(f64),
	String(String),
	Boolean(bool),
	Nil,
}

pub enum Expr {
	Binary(BinaryExpr),
	Grouping(Box<Expr>),
	Literal(LiteralKind),
	Unary(UnaryExpr),
}

// TODO: Perhaps convert these Tokens to SourceTokens

pub struct BinaryExpr {
	pub left: Box<Expr>,
	pub operator: scanner::Token,
	pub right: Box<Expr>,
}

pub struct UnaryExpr {
	pub operator: scanner::Token,
	pub right: Box<Expr>,
}

pub struct Parser {
	tokens: Vec<scanner::SourceToken>,
	index: usize,
}

const EQUALITY_TOKENS: &[scanner::Token] = &[
	scanner::Token::BangEqual,
	scanner::Token::EqualEqual
];

const COMPARISON_TOKENS: &[scanner::Token] = &[
	scanner::Token::Greater,
	scanner::Token::GreaterEqual,
	scanner::Token::Less,
	scanner::Token::LessEqual
];

impl Parser {
	pub fn new(tokens: Vec<scanner::SourceToken>) -> Self {
		Parser { tokens, index: 0 }
	}
	// Token reading
	fn peek_next_token(&self) -> Option<scanner::SourceToken> {
		// Look into this, I have to do it this way to avoid mutable/immutable borrow conflicts.
		// maybe because if I just return `self.tokens.get(self.index)` there's some kind of
		// memory sharing there or smth? Dunno.
		if let Some(token) = self.tokens.get(self.index) {
			Some(token.clone())
		} else {
			None
		}
	}
	fn advance_to_next_token(&mut self) -> Option<&scanner::SourceToken> {
		if let Some(token) = self.tokens.get(self.index) {
			self.index += 1;
			Some(token)
		} else {
			None
		}
	}
	// Rules
	// TODO:? Make a helper function for binaries that just takes a list of the tokens necesary and
	// the next function to match? Might look a bit weird.
	fn equality(&mut self) -> Expr {
		let mut expr = self.comparison();
		while let Some(source_token) = self.peek_next_token() {
			if EQUALITY_TOKENS.contains(&source_token.token) {
				self.advance_to_next_token();
				let operator = source_token.token.clone();
				let right = self.comparison();
				expr = Expr::Binary(BinaryExpr {
					left: Box::new(expr),
					operator,
					right: Box::new(right)
				})
			} else {
				break
			}
		}
		expr
	}
	fn comparison(&mut self) -> Expr {
		let mut expr = self.term();
		while let Some(source_token) = self.peek_next_token() {
			if COMPARISON_TOKENS.contains(&source_token.token) {
				self.advance_to_next_token();
				let operator = source_token.token.clone();
				let right = self.term();
				expr = Expr::Binary(BinaryExpr {
					left: Box::new(expr),
					operator,
					right: Box::new(right)
				})
			} else {
				break
			}
		}
		expr
	}
	fn term(&self) -> Expr {
		todo!();
	}
}