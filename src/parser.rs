use crate::scanner;

pub enum LiteralKind {
	Number(f64),
	String(String),
}

pub enum Expr {
	Binary(Binary),
	Grouping(Box<Expr>),
	Literal(LiteralKind),
	Unary(Unary),
}

// TODO: Perhaps convert these Tokens to SourceTokens

// TODO: Rename these to append Expr to distinguish from enum

pub struct Binary {
	pub left: Box<Expr>,
	pub operator: scanner::Token,
	pub right: Box<Expr>,
}

pub struct Unary {
	pub operator: scanner::Token,
	pub right: Box<Expr>,
}

// Should Literal be a struct containing a value?
