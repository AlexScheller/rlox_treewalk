use crate::language_utilities::enum_variant_equal;
use crate::scanner::{self, WhitespaceKind};

// -----| Expression Grammer |-----
//
// In order of increasing precedence
//
// expression 	-> ternary ;
// ternary		-> equality ( "?" equality ":" equality )* ;
// equality 	-> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison	-> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term			-> factor ( ( "-" | "+" ) factor )* ;
// factor		-> unary ( ( "/" | "*" ) unary )* ;
// unary		-> ( "!" | "-" ) unary | primary ;
// primary		-> NUMBER| | STRING | "true" | "false" | "nil" | "(" expression ")" ;

// TODO: Really think about how clone and copy are to be implemented here.
#[derive(Debug, PartialEq)]
pub enum LiteralKind {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Binary(BinaryExpr),
    Ternary(TernaryExpr),
    Grouping(Box<Expr>),
    Literal(LiteralKind),
    Unary(UnaryExpr),
}

// TODO: Perhaps convert these Tokens to SourceTokens
#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: scanner::Token,
    pub right: Box<Expr>,
}

// We only have one of these, so the operators are implicit
#[derive(Debug)]
pub struct TernaryExpr {
    pub condition: Box<Expr>,
    pub left_result: Box<Expr>,
    pub right_result: Box<Expr>,
}

#[derive(Debug)]
pub struct UnaryExpr {
    pub operator: scanner::Token,
    pub right: Box<Expr>,
}

// -----| Token -> Expression lists |-----

const EQUALITY_TOKENS: &[scanner::Token] = &[scanner::Token::BangEqual, scanner::Token::EqualEqual];

const COMPARISON_TOKENS: &[scanner::Token] = &[
    scanner::Token::Greater,
    scanner::Token::GreaterEqual,
    scanner::Token::Less,
    scanner::Token::LessEqual,
];

const TERM_TOKENS: &[scanner::Token] = &[scanner::Token::Minus, scanner::Token::Plus];

const FACTOR_TOKENS: &[scanner::Token] = &[scanner::Token::Slash, scanner::Token::Star];

const UNARY_TOKENS: &[scanner::Token] = &[scanner::Token::Bang, scanner::Token::Minus];

const TERNARY_TEST_TOKEN: scanner::Token = scanner::Token::QuestionMark;

const TERNARY_BRANCH_TOKEN: scanner::Token = scanner::Token::Colon;

pub struct Parser {
    tokens: Vec<scanner::SourceToken>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<scanner::SourceToken>) -> Self {
        Parser { tokens, index: 0 }
    }
    // Driver
    pub fn parse(&mut self) -> Expr {
        // The tokens provided to the parser may contain whitespace.
        // This seems clunky, we only care about the type, not the value
        let whitespace_exemplar = scanner::Token::Whitespace(WhitespaceKind::Space);
        // I have no idea if this way makes the most sense...
        self.tokens = self
            .tokens
            .drain(..)
            .filter(|source_token| !enum_variant_equal(&source_token.token, &whitespace_exemplar))
            .collect();
        self.expression()
    }
    // Token reading.
    fn peek_next_token(&self) -> Option<scanner::SourceToken> {
        // Look into this, I have to do it this way to avoid mutable/immutable borrow conflicts.
        // maybe because if I just return `self.tokens.get(self.index)` there's some kind of
        // memory sharing there or smth? Dunno.
        if let Some(token) = self.tokens.get(self.index) {
            if token.token == scanner::Token::Eof {
                return None;
            } else {
                return Some(token.clone());
            }
        }
        panic!("Consumed all tokens without encountering EOF");
    }
    fn advance_to_next_token(&mut self) -> Option<&scanner::SourceToken> {
        if let Some(token) = self.tokens.get(self.index) {
            self.index += 1;
            if token.token == scanner::Token::Eof {
                return None;
            } else {
                return Some(token);
            }
        }
        panic!("Consumed all tokens without encountering EOF");
    }
    fn consume_next_token(&mut self, expected_token: scanner::Token) -> &scanner::SourceToken {
        if let Some(next_token) = self.advance_to_next_token() {
            if next_token.token == expected_token {
                return next_token;
            }
            panic!(
                "Expected '{}' after expression, instead found '{}'",
                expected_token, next_token.token
            );
        };
        panic!("Reached end of file while expecting '{}'", expected_token);
    }
    // Rules
    // TODO:? Make a helper function for binaries that just takes a list of the tokens necesary and
    // the next function to match? Might look a bit weird. Also, it may be slightly faster to have
    // them as separate functions. Also, it may become convenient that they are separate later.
    fn expression(&mut self) -> Expr {
        self.ternary()
    }
    fn ternary(&mut self) -> Expr {
        let mut expr = self.equality();
        while let Some(source_token) = self.peek_next_token() {
            if source_token.token == TERNARY_TEST_TOKEN {
                self.advance_to_next_token();
                let left_result = self.equality();
                self.consume_next_token(TERNARY_BRANCH_TOKEN);
                let right_result = self.equality();
                expr = Expr::Ternary(TernaryExpr {
                    condition: Box::new(expr),
                    left_result: Box::new(left_result),
                    right_result: Box::new(right_result),
                })
            } else {
                break;
            }
        }
        expr
    }
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
                    right: Box::new(right),
                })
            } else {
                break;
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
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        expr
    }
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while let Some(source_token) = self.peek_next_token() {
            if TERM_TOKENS.contains(&source_token.token) {
                self.advance_to_next_token();
                let operator = source_token.token.clone();
                let right = self.factor();
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        expr
    }
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while let Some(source_token) = self.peek_next_token() {
            if FACTOR_TOKENS.contains(&source_token.token) {
                self.advance_to_next_token();
                let operator = source_token.token.clone();
                let right = self.unary();
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        expr
    }
    fn unary(&mut self) -> Expr {
        if let Some(source_token) = self.peek_next_token() {
            if UNARY_TOKENS.contains(&source_token.token) {
                self.advance_to_next_token();
                let operator = source_token.token.clone();
                let right = self.unary();
                return Expr::Unary(UnaryExpr {
                    operator,
                    right: Box::new(right),
                });
            }
        }
        self.primary()
    }
    fn primary(&mut self) -> Expr {
        if let Some(source_token) = self.peek_next_token() {
            self.advance_to_next_token();
            match source_token.token {
                scanner::Token::False => Expr::Literal(LiteralKind::Boolean(false)),
                scanner::Token::True => Expr::Literal(LiteralKind::Boolean(true)),
                scanner::Token::Nil => Expr::Literal(LiteralKind::Nil),
                scanner::Token::Number(value) => Expr::Literal(LiteralKind::Number(value)),
                scanner::Token::String(value) => Expr::Literal(LiteralKind::String(value)),
                scanner::Token::LeftParen => {
                    let expr = self.expression();
                    self.consume_next_token(scanner::Token::RightParen);
                    Expr::Grouping(Box::new(expr))
                }
                _ => panic!(
                    "No rule satisfies termination by token: {}",
                    source_token.token
                ),
            }
        } else {
            panic!("Ran out of tokens while satisfying rule")
        }
    }
}
