use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

use crate::errors;
// use crate::language_utilities::enum_variant_equal;
use crate::source_file;

const USE_EXTENDED_UNICODE: bool = true;

// -----| Symbols |-----

type Symbol = String;

#[derive(Debug, Clone, PartialEq)]
pub enum WhitespaceKind {
    Space,
    Tab,
    Return,
    Newline,
}

#[derive(Debug, Clone, PartialEq)]
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
    QuestionMark,
    Colon,
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
    Number(f64),
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
    Comment(String),
    Whitespace(WhitespaceKind),
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Token::LeftParen => String::from("("),
            Token::RightParen => String::from(")"),
            Token::LeftBrace => String::from("{"),
            Token::RightBrace => String::from("}"),
            Token::Comma => String::from(","),
            Token::Dot => String::from("."),
            Token::Minus => String::from("-"),
            Token::Plus => String::from("+"),
            Token::Semicolon => String::from(";"),
            Token::Slash => String::from("/"),
            Token::Star => String::from("*"),
            Token::QuestionMark => String::from("?"),
            Token::Colon => String::from(":"),
            Token::Bang => String::from("!"),
            Token::BangEqual => String::from("!="),
            Token::Equal => String::from("="),
            Token::EqualEqual => String::from("=="),
            Token::Greater => String::from(">"),
            Token::GreaterEqual => String::from(">="),
            Token::Less => String::from("<"),
            Token::LessEqual => String::from("<="),
            Token::Identifier(identifier) => format!("identifier \"{}\"", identifier),
            Token::String(string) => format!("string \"{}\"", string),
            Token::Number(number) => format!("number \"{}\"", number),
            Token::And => String::from("and"),
            Token::Class => String::from("class"),
            Token::Else => String::from("else"),
            Token::False => String::from("false"),
            Token::Fun => String::from("fun"),
            Token::For => String::from("for"),
            Token::If => String::from("if"),
            Token::Nil => String::from("nil"),
            Token::Or => String::from("or"),
            Token::Print => String::from("print"),
            Token::Return => String::from("return"),
            Token::Super => String::from("super"),
            Token::This => String::from("this"),
            Token::True => String::from("true"),
            Token::Var => String::from("var"),
            Token::While => String::from("while"),
            Token::Comment(comment) => format!("comment \"{}\"", comment),
            Token::Whitespace(whitespace) => format!("whitespace {:?}", whitespace),
            Token::Eof => String::from("Eof"),
        };
        write!(f, "{}", value)
    }
}

fn match_keyword(symbol: &str) -> Option<Token> {
    match symbol {
        "and" => Some(Token::And),
        "class" => Some(Token::Class),
        "else" => Some(Token::Else),
        "false" => Some(Token::False),
        "for" => Some(Token::For),
        "fun" => Some(Token::Fun),
        "if" => Some(Token::If),
        "nil" => Some(Token::Nil),
        "or" => Some(Token::Or),
        "print" => Some(Token::Print),
        "return" => Some(Token::Return),
        "super" => Some(Token::Super),
        "this" => Some(Token::This),
        "true" => Some(Token::True),
        "var" => Some(Token::Var),
        "while" => Some(Token::While),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct SourceToken {
    pub token: Token,
    pub location_span: source_file::SourceSpan,
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

// Lol wtf is this. See if this is a performance concern and try to remove it. there's honestly
// probably a way better of doing this.
fn grapheme_to_char(symbol: &str) -> char {
    symbol.to_string().chars().collect::<Vec<char>>()[0]
}

fn is_digit(symbol: &str) -> bool {
    grapheme_to_char(symbol).is_ascii_digit()
}

fn is_alpha(symbol: &str) -> bool {
    let as_char = grapheme_to_char(symbol);
    as_char.is_ascii_alphabetic() || as_char == '_'
}

fn is_alpha_numeric(symbol: &str) -> bool {
    is_alpha(symbol) || is_digit(symbol)
}

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
        while let Some(scan_result) = self.scan_next_token() {
            match scan_result {
                Ok(token) => self.tokens.push(token),
                Err(error) => self.error_log.push(error),
            }
        }
        self.tokens.push(SourceToken {
            token: Token::Eof,
            location_span: self.cursor,
        })
    }
    // Note that this is the only function that will ever "close" the scanning cursor. All other
    // actions only advance it.
    fn scan_next_token(&mut self) -> Option<Result<SourceToken, errors::Error>> {
        if let Some(symbol) = self.consume_next_symbol() {
            let scan_result = match symbol.as_ref() {
                "(" => Ok(Token::LeftParen),
                ")" => Ok(Token::RightParen),
                "{" => Ok(Token::LeftBrace),
                "}" => Ok(Token::RightBrace),
                "," => Ok(Token::Comma),
                "." => Ok(Token::Dot),
                "-" => Ok(Token::Minus),
                "+" => Ok(Token::Plus),
                ";" => Ok(Token::Semicolon),
                "*" => Ok(Token::Star),
                "?" => Ok(Token::QuestionMark),
                ":" => Ok(Token::Colon),
                "!" => {
                    if self.match_next_symbol("=") {
                        Ok(Token::BangEqual)
                    } else {
                        Ok(Token::Bang)
                    }
                }
                "=" => {
                    if self.match_next_symbol("=") {
                        Ok(Token::EqualEqual)
                    } else {
                        Ok(Token::Equal)
                    }
                }
                "<" => {
                    if self.match_next_symbol("=") {
                        Ok(Token::LessEqual)
                    } else {
                        Ok(Token::Less)
                    }
                }
                ">" => {
                    if self.match_next_symbol("=") {
                        Ok(Token::GreaterEqual)
                    } else {
                        Ok(Token::Greater)
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
                        Ok(Token::Comment(content))
                    } else {
                        Ok(Token::Slash)
                    }
                }
                // --- Whitespace ---
                " " => Ok(Token::Whitespace(WhitespaceKind::Space)),
                "\r" => Ok(Token::Whitespace(WhitespaceKind::Return)),
                "\t" => Ok(Token::Whitespace(WhitespaceKind::Tab)),
                "\n" => Ok(Token::Whitespace(WhitespaceKind::Newline)),
                "\"" => self.consume_string(),
                digit if is_digit(digit) => self.consume_number(),
                identifier if is_alpha(identifier) => self.consume_identifier(),
                _ => Err(errors::Error {
                    kind: errors::ErrorKind::Scanning,
                    description: errors::ErrorDescription {
                        subject: Some(String::from(symbol)),
                        location: self.cursor,
                        description: String::from("Unexpected character"),
                    },
                }),
            };
            let ret = match scan_result {
                Ok(token) => {
                    let location_span = self.cursor;
                    Some(Ok(SourceToken {
                        token,
                        location_span,
                    }))
                }
                Err(error) => Some(Err(error)),
            };
            self.cursor.close();
            ret
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
    fn peek_next_symbol_twice(&self) -> Option<Symbol> {
        if let Some(curr) = self.source.get(self.cursor.end.index + 1) {
            Some(curr.to_string())
        } else {
            None
        }
    }
    fn consume_string(&mut self) -> Result<Token, errors::Error> {
        while let Some(symbol) = self.peek_next_symbol() {
            self.cursor.end.increment(&symbol);
            if symbol == "\"" {
                let string_value = self.source_substring(self.cursor);
                return Ok(Token::String(
                    string_value[1..string_value.len() - 1].to_string(),
                ));
            }
        }
        let error_string = self.source_substring(self.cursor);
        Err(errors::Error {
            kind: errors::ErrorKind::Scanning,
            description: errors::ErrorDescription {
                subject: Some(error_string),
                location: self.cursor,
                description: String::from("Unterminated String"),
            },
        })
    }
    fn source_substring(&self, cursor: source_file::SourceSpan) -> String {
        self.source[cursor.start.index..cursor.end.index].join("")
    }
    // TODO: This function is crunchy as hell, also refactor peeking? I think this technically
    // allows numbers like "10."
    // TODO: Something seems fishy that this doesn't return any errors...
    fn consume_number(&mut self) -> Result<Token, errors::Error> {
        // Consume all digits until you run out.
        // TODO: Duplicated code.
        while let Some(symbol) = self.peek_next_symbol() {
            if is_digit(&symbol) {
                self.consume_next_symbol();
            } else {
                break;
            }
        }
        // See if there's a decimal point, if so, continue consuming digits until you run out.
        if let Some(symbol) = self.peek_next_symbol() {
            if symbol == "." {
                if let Some(symbol) = self.peek_next_symbol_twice() {
                    if is_digit(&symbol) {
                        // Consume the "."
                        self.consume_next_symbol();
                        // TODO: Duplicated Code
                        while let Some(symbol) = self.peek_next_symbol() {
                            if is_digit(&symbol) {
                                self.consume_next_symbol();
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
        let value = self
            .source_substring(self.cursor)
            .parse::<f64>()
            .expect("Internal error parsing float!");
        Ok(Token::Number(value))
    }
    // TODO: Another one that doesn't return errors??
    fn consume_identifier(&mut self) -> Result<Token, errors::Error> {
        while let Some(symbol) = self.peek_next_symbol() {
            if is_alpha_numeric(&symbol) {
                self.consume_next_symbol();
            } else {
                break;
            }
        }
        let value = self.source_substring(self.cursor);
        if let Some(keyword) = match_keyword(&value) {
            Ok(keyword)
        } else {
            Ok(Token::Identifier(value))
        }
    }
}

impl errors::ErrorLoggable for Scanner {
    fn error_log(&self) -> &errors::ErrorLog {
        &self.error_log
    }
}
