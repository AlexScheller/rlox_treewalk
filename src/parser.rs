use crate::scanner;

pub enum Expr {
	Binary(Binary),
	Literal(String),
}

pub struct Binary {
	pub left: Box<Expr>,
	pub operator: scanner::SourceToken,
	pub right: Box<Expr>,
}
