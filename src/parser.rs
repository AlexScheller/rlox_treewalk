use crate::errors;
use crate::language_utilities::enum_variant_equal;
use crate::scanner::{self, WhitespaceKind};

// -----| Syntax Grammer |-----
//
// program  -> declaration* EOF ;

// -----| Declaration Grammar |-----
//
// declaration  -> varDecl | statement ;
// varDecl      -> "var" IDENTIFIER ( "=" expression )? ";" ;

// -----| Statement Grammar |-----
//
// statement    -> epxrStmt | print Stmt ;
// exprStmt     -> expression ";" ;
// printStmt    -> "print" expression ";" ;

const STATEMENT_BEGINNING_TOKENS: &[scanner::Token] = &[
    scanner::Token::Class,
    scanner::Token::For,
    scanner::Token::Fun,
    scanner::Token::If,
    scanner::Token::Print,
    scanner::Token::Return,
    scanner::Token::Var,
    scanner::Token::While,
];

// TODO: Can these be simplified?
pub enum Stmt {
    Expression(ExprStmt),
    Print(PrintStmt),
    Var(VarStmt),
}

pub struct ExprStmt {
    pub expression: Expr,
}

// TODO: Get rid of this as soon as you have a standard library. This is a bootstrapping thing.
pub struct PrintStmt {
    pub expression: Expr,
}

pub struct VarStmt {
    pub name: scanner::Identifier,
    pub initializer: Option<Expr>,
}

// -----| Expression Grammer |-----
//
// In increasing order of precedence
//
// expression  -> ternary ;
// ternary     -> equality ( "?" equality ":" equality )* ;
// equality    -> comparison ( ( "!=" | "==" ) comparison )* ;
// comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term        -> factor ( ( "-" | "+" ) factor )* ;
// factor      -> unary ( ( "/" | "*" ) unary )* ;
// unary       -> ( "!" | "-" ) unary | primary ;
// primary     -> NUMBER| | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;

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
    Unary(UnaryExpr),
    Literal(LiteralKind),
    // Variable(scanner::Identifier),
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

// -----| Token Exemplars |-----

// TODO: Find out a more rustish way of handling the case where you need to compare the type of enum
// but not the value. Right now I just create "fake" ones as examples.

const WHITESPACE_EXEMPLAR: scanner::Token = scanner::Token::Whitespace(WhitespaceKind::Space);

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
        // TODO: I have no idea if this is the best way to filter this vector.
        self.tokens = self
            .tokens
            .drain(..)
            .filter(|source_token| !enum_variant_equal(&source_token.token, &WHITESPACE_EXEMPLAR))
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
        if let Some(_) = self.peek_next_token() {
            Some(self.declaration())
        } else {
            None
        }
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
    fn match_then_consume(&mut self, token: scanner::Token, target: scanner::Token) -> bool {
        if token == target {
            self.deprecated_advance_token_index();
            true
        } else {
            false
        }
    }
    // TODO: ~~Reconcile these two~~ Actually only the second should be used. There's only one
    // instance of a function actually unwraping the Option.
    fn deprecated_advance_token_index(&mut self) -> Option<scanner::SourceToken> {
        if let Some(token) = self.tokens.get(self.index) {
            self.index += 1;
            if token.token == scanner::Token::Eof {
                return None;
            } else {
                return Some(token.clone());
            }
        }
        panic!("`advance_next_token` Consumed all tokens without encountering EOF");
    }
    fn advance_token_index(&mut self) -> Result<scanner::SourceToken, errors::Error> {
        if let Some(token) = self.tokens.get(self.index) {
            self.index += 1;
            // TODO Some kind of error for reaching Eof?
            return Ok(token.clone());
        }
        Err(errors::Error {
            kind: errors::ErrorKind::Parsing,
            description: errors::ErrorDescription {
                subject: None,
                location: None,
                description: String::from("Consumed all tokens without encountering EOF"),
            },
        })
    }
    fn consume_next_token(
        &mut self,
        expected_token: scanner::Token,
    ) -> Result<scanner::SourceToken, errors::Error> {
        if let Some(next_token) = self.peek_next_token() {
            self.deprecated_advance_token_index();
            if enum_variant_equal(&next_token.token, &expected_token) {
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
    fn synchronize_to_statement_boundary(&mut self) {
        while let Some(source_token) = self.deprecated_advance_token_index() {
            if self.previous_token().token == scanner::Token::Semicolon
                || STATEMENT_BEGINNING_TOKENS.contains(&source_token.token)
            {
                break;
            }
        }
    }
    // --- Statement Rules ---
    fn declaration(&mut self) -> Result<Stmt, errors::Error> {
        if let Some(source_token) = self.peek_next_token() {
            let res = if self.match_then_consume(source_token.token, scanner::Token::Var) {
                self.var_declaration()
            } else {
                self.statement()
            };
            return match res {
                Ok(stmt) => Ok(stmt),
                Err(error) => {
                    self.synchronize_to_statement_boundary();
                    Err(error)
                }
            };
        }
        // Should this be here?
        panic!("Attempted to parse declartion with no tokens left.");
    }
    fn var_declaration(&mut self) -> Result<Stmt, errors::Error> {
        // TODO: Find out a way to make this a constant. This is a real bummer, or find out if you
        // can pass in just the type of the enum without constructing it.
        let IDENTIFIER_EXEMPLAR = scanner::Token::Identifier(String::from("example"));
        // Woof this deconstruction is a mouthful.
        if let scanner::SourceToken {
            token: scanner::Token::Identifier(name),
            ..
        } = self.consume_next_token(IDENTIFIER_EXEMPLAR)?
        {
            let mut initializer = None;
            let source_token = self.advance_token_index()?;
            if self.match_then_consume(source_token.token, scanner::Token::Equal) {
                initializer = Some(self.expression()?);
            }
            self.consume_next_token(scanner::Token::Semicolon)?;
            return Ok(Stmt::Var(VarStmt { name, initializer }));
        };
        // TODO: Find out a better way to structure this. It would be nice if rust had type
        // narrowing from function returns.
        panic!("`consume_next_token` has to be broken for this to be reachable");
    }
    fn statement(&mut self) -> Result<Stmt, errors::Error> {
        if let Some(source_token) = self.peek_next_token() {
            if self.match_then_consume(source_token.token, scanner::Token::Print) {
                return self.print_statement();
            }
        }
        // Note, it seems absurd to let control fall through into `expression_statement()` after we
        // *know* that there isn't a token to consume, but the correct error *will* propagate when
        // it reaches the bottom of the call stack. This is therefore not technically wrong, but
        // could certainly be optimized. There's a certain elegance to it, but maybe that's wrong.
        // This is also how it works in the book, for whatever that's worth.
        self.expression_statement()
    }
    fn print_statement(&mut self) -> Result<Stmt, errors::Error> {
        let expression = self.expression()?;
        self.consume_next_token(scanner::Token::Semicolon)?;
        Ok(Stmt::Print(PrintStmt { expression }))
    }
    fn expression_statement(&mut self) -> Result<Stmt, errors::Error> {
        let expression = self.expression()?;
        self.consume_next_token(scanner::Token::Semicolon)?;
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
                self.deprecated_advance_token_index();
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
                self.deprecated_advance_token_index();
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
                self.deprecated_advance_token_index();
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
                self.deprecated_advance_token_index();
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
                self.deprecated_advance_token_index();
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
                self.deprecated_advance_token_index();
                let operator = source_token.token.clone();
                let right = self.unary()?;
                return Ok(Expr::Unary(UnaryExpr {
                    operator,
                    right: Box::new(right),
                }));
            }
        }
        // Note, See the note above in `statement()` regarding calling another function after we
        // know that we are out of tokens.
        self.primary()
    }
    fn primary(&mut self) -> Result<Expr, errors::Error> {
        if let Some(source_token) = self.peek_next_token() {
            self.deprecated_advance_token_index();
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
