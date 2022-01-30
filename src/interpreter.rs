use crate::parser::{BinaryExpr, Expr, LiteralKind, TernaryExpr, UnaryExpr};
use crate::scanner::Token;

// // Rust's native method of runtime introspection is not recomended for anything other than debugging.
// trait TypeInfoable {
// 	fn type(&self) ->
// }

trait Boolable {
    fn to_bool_option(&self) -> Option<bool>;
}

impl Boolable for LiteralKind {
    fn to_bool_option(&self) -> Option<bool> {
        match self {
            LiteralKind::Boolean(value) => Some(*value),
            LiteralKind::Nil => Some(false),
            LiteralKind::Number(_) => None,
            LiteralKind::String(_) => None,
        }
    }
}

fn is_truthy(investigatee: LiteralKind) -> bool {
    if let Some(value) = investigatee.to_bool_option() {
        value
    } else {
        false
    }
}

// For now, just relying on PartialEq should be good enough. In the future, this may nee to be
// changed, which is why we use this function to wrap the equality check.
fn is_equal(a: LiteralKind, b: LiteralKind) -> bool {
    a == b
    // Maybe in the future we want to prevent comparisons between types that can never be
    // equivilent. Certianly I have no interest in equality checks suceeding between heterogenus
    // types of the kind JS allows.
    // if enum_variant_equal(&a, &b) {
    //     return a == b;
    // }
    // panic!("Illegal equality comparison of operands")
}

pub fn interpret_expression(expr: Expr) -> LiteralKind {
    let ret = match expr {
        Expr::Literal(literal) => literal,
        Expr::Grouping(group) => interpret_expression(*group),
        Expr::Unary(unary) => interpret_unary(unary),
        Expr::Binary(binary) => interpret_binary(binary),
        Expr::Ternary(ternary) => interpret_ternary(ternary),
    };
    ret
}

// We've broken up the different expression categories, but we could also break up the individual
// operand handlers. Also, there are many checks in these functions that could themselves be
// functions, but we are leaving them expanded for now for flexibility. The error reporting can also be made way simpler
fn interpret_unary(UnaryExpr { operator, right }: UnaryExpr) -> LiteralKind {
    let right_literal = interpret_expression(*right);
    match operator {
        Token::Minus => {
            if let LiteralKind::Number(value) = right_literal {
                return LiteralKind::Number(-value);
            } else {
                panic!(
                    "Illegal operand for unary '{}' expression: {:?}",
                    Token::Minus,
                    right_literal
                )
            }
        }
        Token::Bang => {
            match right_literal {
                // following two lines are technically redundant. Could be better
                LiteralKind::Nil | LiteralKind::Boolean(_) => {
                    return LiteralKind::Boolean(!is_truthy(right_literal));
                }
                _ => panic!(
                    "Illegal operand for unary '{}' expression: {:?}",
                    Token::Bang,
                    right_literal
                ),
            }
        }
        // Note, I think this should theoretically be impossible. The parser should catch these
        // earlier?
        _ => panic!("Illegal operator for unary expression: {}", operator),
    }
}

// Right now, we're checking if both operands are numeric for every single operator, but also we
// only support numeric operations (the book allows string concatenation but I don't). We could
// thus check for numeric once at the beginning, but that would have to be refactored if we ever
// wanted to support non-numeric binary operations.
fn interpret_binary(
    BinaryExpr {
        left,
        operator,
        right,
    }: BinaryExpr,
) -> LiteralKind {
    let left_literal = interpret_expression(*left);
    let right_literal = interpret_expression(*right);
    match operator {
        Token::Minus => {
            // TODO: Find a nicer looking way of doing this. I tried double extracting from a tuple,
            // but the values had to be `move`d into the tuple, so they couldn't be used in the
            // panic string format.
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Number(left_value - right_value);
                }
            }
            // Hmm, technically we don't say which one is wrong (or maybe both) but the user can
            // probably figure it out if we print both.
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::Minus,
                left_literal,
                Token::Minus,
                right_literal
            )
        }
        Token::Slash => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Number(left_value / right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::Slash,
                left_literal,
                Token::Slash,
                right_literal
            )
        }
        Token::Star => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Number(left_value * right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::Star,
                left_literal,
                Token::Star,
                right_literal
            )
        }
        Token::Plus => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Number(left_value + right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::Plus,
                left_literal,
                Token::Plus,
                right_literal
            )
        }
        Token::Greater => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Boolean(left_value > right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::Greater,
                left_literal,
                Token::Greater,
                right_literal
            )
        }
        Token::GreaterEqual => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Boolean(left_value >= right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::GreaterEqual,
                left_literal,
                Token::GreaterEqual,
                right_literal
            )
        }
        Token::Less => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Boolean(left_value < right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::Less,
                left_literal,
                Token::Less,
                right_literal
            )
        }
        Token::LessEqual => {
            if let LiteralKind::Number(left_value) = left_literal {
                if let LiteralKind::Number(right_value) = right_literal {
                    return LiteralKind::Boolean(left_value <= right_value);
                }
            }
            panic!(
                "Illegal operand for binary '{}' expression: {:?} {} {:?}",
                Token::LessEqual,
                left_literal,
                Token::LessEqual,
                right_literal
            )
        }
        Token::BangEqual => return LiteralKind::Boolean(!is_equal(left_literal, right_literal)),
        Token::EqualEqual => return LiteralKind::Boolean(is_equal(left_literal, right_literal)),
        _ => panic!("Illegal operator for binary expression: {}", operator),
    }
}

fn interpret_ternary(
    TernaryExpr {
        condition,
        left_result,
        right_result,
    }: TernaryExpr,
) -> LiteralKind {
    let condition_literal = interpret_expression(*condition);
    // Note, we could check if this is "truthy" instead of an explicit boolean check, but I'd prefer
    // not to.
    if let LiteralKind::Boolean(condition_value) = condition_literal {
        let left_result_literal = interpret_expression(*left_result);
        let right_result_literal = interpret_expression(*right_result);
        if condition_value {
            left_result_literal
        } else {
            right_result_literal
        }
    } else {
        panic!(
            "Non boolean type used as condition in ternary: {:?}",
            condition_literal
        );
    }
}
