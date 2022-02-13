use crate::errors;
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
    /// The actual index we use to iterate throuh the tokens.
    index: usize,
    // cursor: source_file::SourceSpan, // Should this be used?
    // error_log: errors::ErrorLog, // TODO: Use this. right now, only one expression can be
    // evaulated, so there's no need for multiple errors.
}

impl Parser {
    pub fn new(tokens: Vec<scanner::SourceToken>) -> Self {
        Parser {
            tokens,
            index: 0,
            // cursor: source_file::SourceSpan::new(),
            // error_log: errors::ErrorLog::new(),
        }
    }
    // Driver
    pub fn parse(&mut self) -> Result<Expr, errors::Error> {
        // The tokens provided to the parser may contain whitespace.
        // This seems clunky, we only care about the type, not the value
        let whitespace_exemplar = scanner::Token::Whitespace(WhitespaceKind::Space);
        // I have no idea if this way makes the most sense...
        self.tokens = self
            .tokens
            .drain(..)
            .filter(|source_token| !enum_variant_equal(&source_token.token, &whitespace_exemplar))
            .collect();
        // TODO: Return a list of expressions. If an expression finishes evaulating, and there are
        // still more tokens, check for a semicolon then continue appending evaluated expressions
        self.expression()
    }
    // Token reading.
    // TODO: Reconcile the fact that we nominally deal with "previous" and "next" tokens in these
    // functions, but not "current" tokens. I guess that's not a big deal, the "current" tokens are
    // only ever current within the context of a given function?
    fn peek_next_token(&self) -> Option<scanner::SourceToken> {
        // Look into this, I have to do it this way to avoid mutable/immutable borrow conflicts.
        // maybe because if I just return `self.tokens.get(self.index)` there's some kind of
        // memory sharing there or smth? Dunno.

        // We panic, rather than returning an error, because the Eof sentinal should have been
        // appended to the token list *by the scanner*.
        let token = self
            .tokens
            .get(self.index)
            .expect("Consumed all tokens without encountering EOF");
        if token.token == scanner::Token::Eof {
            return None;
        } else {
            return Some(token.clone());
        }
    }
    fn advance_token_index(&mut self) -> Option<&scanner::SourceToken> {
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
    fn consume_next_token(
        &mut self,
        expected_token: scanner::SourceToken,
    ) -> Result<&scanner::SourceToken, errors::Error> {
        if let Some(next_token) = self.advance_token_index() {
            if next_token.token == expected_token.token {
                return Ok(next_token);
            }
            return Err(errors::Error {
                kind: errors::ErrorKind::Parsing,
                description: errors::ErrorDescription {
                    subject: None,
                    location: Some(next_token.location_span),
                    description: format!(
                        "Expected '{}' after expression, instead found '{}'",
                        expected_token.token, next_token.token
                    ),
                },
            });
        };
        Err(errors::Error {
            kind: errors::ErrorKind::Parsing,
            description: errors::ErrorDescription {
                subject: None,
                location: Some(expected_token.location_span),
                description: format!(
                    "Reached end of file while expecting '{}'",
                    expected_token.token
                ),
            },
        })
    }
    // Maybe would be better to use a cursor?
    fn previous_token(&self) -> scanner::SourceToken {
        if self.index > 0 {
            return self.tokens.get(self.index - 1).unwrap().clone();
        }
        panic!("Attempted to read previous token while at index 0");
    }
    // TODO: This one will take some thinking. The idea is to run the token index to the next
    // statement boundary, and begin parsing again.
    // fn synchronize_to_statement_boundary(&self) {
    //     while let Some(source_token) = self.advance_token_index() {
    //         match source_token.token {
    //             scanner::Token::Semicolon => break,
    //             // scanner::Token::Class | scanner::Token::For |
    //             // scanner::Token::Fun | scanner::Token::If |
    //             // scanner::Token::Print | scanner::Token::Return |
    //             // scanner::Token::Var | scanner::Token::While => break
    //         }
    //     }
    // }
    // Rules
    // TODO:? Make a helper function for binaries that just takes a list of the tokens necesary and
    // the next function to match? Might look a bit weird. Also, it may be slightly faster to have
    // them as separate functions. Also, it may become convenient that they are separate later.

    // TODO: Read up on error handling
    fn expression(&mut self) -> Result<Expr, errors::Error> {
        self.ternary()
    }
    fn ternary(&mut self) -> Result<Expr, errors::Error> {
        let mut expr = self.equality()?;
        while let Some(source_token) = self.peek_next_token() {
            if source_token.token == TERNARY_TEST_TOKEN {
                self.advance_token_index();
                let left_result = self.equality()?;
                // TODO: Deal with this. I'm creating a synthetic token with the location of the
                // current token, but the value of the token I'm seeking. This is "correct" when it
                // comes to locating the error, but still feels wrong.
                self.consume_next_token(scanner::SourceToken {
                    token: TERNARY_BRANCH_TOKEN,
                    ..source_token
                })?;
                let right_result = self.equality()?;
                expr = Expr::Ternary(TernaryExpr {
                    condition: Box::new(expr),
                    left_result: Box::new(left_result),
                    right_result: Box::new(right_result),
                })
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn equality(&mut self) -> Result<Expr, errors::Error> {
        let mut expr = self.comparison()?;
        while let Some(source_token) = self.peek_next_token() {
            if EQUALITY_TOKENS.contains(&source_token.token) {
                self.advance_token_index();
                let operator = source_token.token.clone();
                let right = self.comparison()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn comparison(&mut self) -> Result<Expr, errors::Error> {
        let mut expr = self.term()?;
        while let Some(source_token) = self.peek_next_token() {
            if COMPARISON_TOKENS.contains(&source_token.token) {
                self.advance_token_index();
                let operator = source_token.token.clone();
                let right = self.term()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<Expr, errors::Error> {
        let mut expr = self.factor()?;
        while let Some(source_token) = self.peek_next_token() {
            if TERM_TOKENS.contains(&source_token.token) {
                self.advance_token_index();
                let operator = source_token.token.clone();
                let right = self.factor()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, errors::Error> {
        let mut expr = self.unary()?;
        while let Some(source_token) = self.peek_next_token() {
            if FACTOR_TOKENS.contains(&source_token.token) {
                self.advance_token_index();
                let operator = source_token.token.clone();
                let right = self.unary()?;
                expr = Expr::Binary(BinaryExpr {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                })
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, errors::Error> {
        if let Some(source_token) = self.peek_next_token() {
            if UNARY_TOKENS.contains(&source_token.token) {
                self.advance_token_index();
                let operator = source_token.token.clone();
                let right = self.unary()?;
                return Ok(Expr::Unary(UnaryExpr {
                    operator,
                    right: Box::new(right),
                }));
            }
        }
        self.primary()
    }
    fn primary(&mut self) -> Result<Expr, errors::Error> {
        if let Some(source_token) = self.peek_next_token() {
            self.advance_token_index();
            match source_token.token {
                scanner::Token::False => Ok(Expr::Literal(LiteralKind::Boolean(false))),
                scanner::Token::True => Ok(Expr::Literal(LiteralKind::Boolean(true))),
                scanner::Token::Nil => Ok(Expr::Literal(LiteralKind::Nil)),
                scanner::Token::Number(value) => Ok(Expr::Literal(LiteralKind::Number(value))),
                scanner::Token::String(value) => Ok(Expr::Literal(LiteralKind::String(value))),
                scanner::Token::LeftParen => {
                    let expr = self.expression()?;
                    self.consume_next_token(scanner::SourceToken {
                        token: scanner::Token::RightParen,
                        ..source_token
                    })?;
                    Ok(Expr::Grouping(Box::new(expr)))
                }
                _ => Err(errors::Error {
                    kind: errors::ErrorKind::Parsing,
                    description: errors::ErrorDescription {
                        subject: None,
                        location: Some(source_token.location_span),
                        description: format!(
                            "Expected value or expression, found '{}'",
                            source_token.token
                        ), // TODO: Better wording?
                    },
                }),
            }
        } else {
            Err(errors::Error {
                kind: errors::ErrorKind::Parsing,
                description: errors::ErrorDescription {
                    subject: None,
                    location: Some(self.previous_token().location_span),
                    description: String::from("Ran out of tokens while satisfying rule"),
                },
            })
        }
    }
}
