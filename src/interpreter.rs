use crate::parser;
use crate::parser::LiteralKind;
use crate::scanner;

// // Rusts method of runtime introspection is not recomended for anything other than debugging.
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
            LiteralKind::Number(value) => None, // For now... mwahahaha
            LiteralKind::String(value) => None, // Same as above
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

// TODO: This would actually be nice to break up a bit...
pub fn interpret(expr: parser::Expr) -> parser::LiteralKind {
    let ret = match expr {
        parser::Expr::Literal(literal) => literal,
        parser::Expr::Grouping(group) => interpret(*group),
        parser::Expr::Unary(parser::UnaryExpr { operator, right }) => {
            let right = interpret(*right);
            match operator {
                scanner::Token::Minus => {
                    if let LiteralKind::Number(value) = right {
                        return LiteralKind::Number(-value);
                    } else {
                        panic!("Illegal operand for unary '-' expression: {:?}", right)
                    }
                }
                scanner::Token::Bang => {
                    match right {
                        // following two lines are technically redundant. Could be better
                        LiteralKind::Nil | LiteralKind::Boolean(_) => {
                            return LiteralKind::Boolean(!is_truthy(right));
                        }
                        _ => panic!("Illegal operand for unary '!' expression: {:?}", right),
                    }
                }
                _ => panic!("Illegal operator for unary expression: {}", operator),
            }
        }
    };
    ret
}
