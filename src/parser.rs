use crate::errors;
use crate::language_utilities::enum_variant_equal;
use crate::scanner::{self, WhitespaceKind};

// -----| Expression Grammer |-----
//
// In increasing order of increasing precedence
//
// expression  -> ternary ;
// ternary     -> equality ( "?" equality ":" equality )* ;
// equality    -> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term        -> factor ( ( "-" | "+" ) factor )* ;
// factor      -> unary ( ( "/" | "*" ) unary )* ;
// unary       -> ( "!" | "-" ) unary | primary ;
// primary     -> NUMBER| | STRING | "true" | "false" | "nil" | "(" expression ")" ;

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

// -----| Statement Grammar |-----
//
// In increasing order of increasing precedence
//
// program    -> statement* EOF ;
// statement  -> epxrStmt | print Stmt ;
// exprStmt   -> expression ";" ;
// printStmt  -> "print" expression ";" ;

const STATEMENT_BOUNDARY_TOKEN: scanner::Token = scanner::Token::Semicolon;

// TODO: Can these be simplified?
pub enum Stmt {
    Expression(ExprStmt),
    Print(PrintStmt),
}

pub struct ExprStmt {
    pub expression: Expr,
}

// TODO: Get rid of this as soon as you have a standard library. This is a bootstrapping thing.
pub struct PrintStmt {
    pub expression: Expr,
}

// -----| Parsing |-----

pub struct Parser {
    tokens: Vec<scanner::SourceToken>,
    /// The actual index we use to iterate throuh the tokens.
    index: usize,
    // cursor: source_file::SourceSpan, // Should this be used?
    error_log: errors::ErrorLog,
}

impl Parser {
    pub fn new(tokens: Vec<scanner::SourceToken>) -> Self {
        Parser {
            tokens,
            index: 0,
            // cursor: source_file::SourceSpan::new(),
            error_log: errors::ErrorLog::new(),
        }
    }
    // --- Drivers ---
    // TODO: Clean this up so that the parser doesn't need to strip its own whitespace?
    pub fn parse(&mut self) -> Vec<Stmt> {
        // The tokens provided to the parser may contain whitespace.
        // This seems clunky, we only care about the type, not the value
        let whitespace_exemplar = scanner::Token::Whitespace(WhitespaceKind::Space);
        // I have no idea if this way makes the most sense...
        self.tokens = self
            .tokens
            .drain(..)
            .filter(|source_token| !enum_variant_equal(&source_token.token, &whitespace_exemplar))
            .collect();
        // Begin parsing statements
        let mut statements: Vec<Stmt> = Vec::new();
        while let Some(parse_result) = self.parse_next_statement() {
            match parse_result {
                Ok(statement) => statements.push(statement),
                Err(error) => self.error_log.push(error),
            }
        }
        statements
    }
    fn parse_next_statement(&mut self) -> Option<Result<Stmt, errors::Error>> {
        self.statement()
    }
    // --- Token Reading ---
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
            .expect("`peek_next_token` Consumed all tokens without encountering EOF");
        if token.token == scanner::Token::Eof {
            return None;
        } else {
            return Some(token.clone());
        }
    }
    fn advance_token_index(&mut self) -> Option<scanner::SourceToken> {
        if let Some(token) = self.tokens.get(self.index) {
            self.index += 1;
            if token.token == scanner::Token::Eof {
                return None;
            } else {
                return Some(token.clone());
            }
        }
        panic!("`advance_next_token` Consumed all tokens without encountering EOF in");
    }
    fn consume_next_token(
        &mut self,
        expected_token: scanner::Token,
    ) -> Result<scanner::SourceToken, errors::Error> {
        if let Some(next_token) = self.peek_next_token() {
            self.advance_token_index();
            if next_token.token == expected_token {
                return Ok(next_token);
            }
            return Err(errors::Error {
                kind: errors::ErrorKind::Parsing,
                description: errors::ErrorDescription {
                    subject: None,
                    location: Some(next_token.location_span),
                    description: format!(
                        "Expected '{}' after expression, instead found '{}'",
                        expected_token, next_token.token
                    ),
                },
            });
        };
        Err(errors::Error {
            kind: errors::ErrorKind::Parsing,
            description: errors::ErrorDescription {
                subject: None,
                location: None,
                description: format!("Reached end of file while expecting '{}'", expected_token),
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
    // --- Statement Rules ---
    fn statement(&mut self) -> Option<Result<Stmt, errors::Error>> {
        if let Some(source_token) = self.peek_next_token() {
            let result = match source_token.token {
                scanner::Token::Print => {
                    // the print token won't be matched while parsing expressions, so we must
                    // consume it here.
                    self.advance_token_index();
                    self.print_statement()
                }
                _ => self.expression_statement(),
            };
            return Some(result);
        }
        None // Hmm, is this right?
    }
    fn print_statement(&mut self) -> Result<Stmt, errors::Error> {
        let expression = self.expression()?;
        self.consume_next_token(STATEMENT_BOUNDARY_TOKEN)?;
        Ok(Stmt::Print(PrintStmt { expression }))
    }
    fn expression_statement(&mut self) -> Result<Stmt, errors::Error> {
        let expression = self.expression()?;
        self.consume_next_token(STATEMENT_BOUNDARY_TOKEN)?;
        Ok(Stmt::Expression(ExprStmt { expression }))
    }
    // --- Expression Rules ---
    // TODO:? Make a helper function for binaries that just takes a list of the tokens necesary and
    // the next function to match? Might look a bit weird. Also, it may be slightly faster to have
    // them as separate functions. Also, it may become convenient that they are separate later.
    fn expression(&mut self) -> Result<Expr, errors::Error> {
        self.ternary()
    }
    fn ternary(&mut self) -> Result<Expr, errors::Error> {
        let mut expr = self.equality()?;
        while let Some(source_token) = self.peek_next_token() {
            if source_token.token == TERNARY_TEST_TOKEN {
                self.advance_token_index();
                let left_result = self.equality()?;
                self.consume_next_token(TERNARY_BRANCH_TOKEN)?;
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
                    self.consume_next_token(scanner::Token::RightParen)?;
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
                    description: String::from("Ran out of tokens while satisfying expression rule"),
                },
            })
        }
    }
}

// TODO: I think this can actually be done generically in errors.rs, and handled simply by importing.
impl errors::ErrorLoggable for Parser {
    fn error_log(&self) -> &errors::ErrorLog {
        &self.error_log
    }
}
