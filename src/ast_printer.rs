use crate::parser;

pub fn expr_to_ast_string(expression: &parser::Expr) -> String {
    let ret = match expression {
        parser::Expr::Binary(expr) => {
            format!(
                "({} {} {})",
                expr.operator,
                expr_to_ast_string(&expr.left),
                expr_to_ast_string(&expr.right)
            )
        }
        parser::Expr::Ternary(expr) => {
            format!(
                "({} ? {} : {})",
                expr_to_ast_string(&expr.condition),
                expr_to_ast_string(&expr.left_result),
                expr_to_ast_string(&expr.right_result),
            )
        }
        parser::Expr::Grouping(expr) => {
            format!("(group {})", expr_to_ast_string(&expr))
        }
        parser::Expr::Literal(kind) => match kind {
            parser::LiteralKind::Number(number) => number.to_string(),
            parser::LiteralKind::String(string) => string.to_string(),
            parser::LiteralKind::Boolean(boolean) => boolean.to_string(),
            parser::LiteralKind::Nil => String::from("nil"),
        },
        parser::Expr::Unary(expr) => {
            format!("({} {})", expr.operator, expr_to_ast_string(&expr.right))
        } // parser::Expr::Variable(expr) => {

          // }
    };
    ret
}

pub fn stmt_to_ast_string(statement: &parser::Stmt) -> String {
    let ret = match statement {
        parser::Stmt::Expression(stmt) => {
            format!(
                "Expression Statement: {}",
                expr_to_ast_string(&stmt.expression)
            )
        }
        parser::Stmt::Print(stmt) => {
            format!("Print Statement: {}", expr_to_ast_string(&stmt.expression),)
        }
        parser::Stmt::Var(stmt) => {
            let initilizer_string = if let Some(initializer) = &stmt.initializer {
                format!(" = {}", expr_to_ast_string(initializer))
            } else {
                String::from("")
            };
            format!("Variable Statement: {}{}", stmt.name, initilizer_string)
        }
    };
    ret
}
