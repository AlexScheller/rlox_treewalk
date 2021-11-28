use crate::parser;

pub fn expr_to_ast_string(expression: parser::Expr) -> String {
	let ret = match expression {
		parser::Expr::Binary(expr) => {
			format!(
				"({} {} {})",
				expr.operator,
				expr_to_ast_string(*expr.left),
				expr_to_ast_string(*expr.right)
			)
		}
		parser::Expr::Grouping(expr) => {
			format!("(group {})", expr_to_ast_string(*expr))
		}
		parser::Expr::Literal(kind) => match kind {
			parser::LiteralKind::Number(number) => number.to_string(),
			parser::LiteralKind::String(string) => string,
		},
		parser::Expr::Unary(expr) => {
			format!("({} {})", expr.operator, expr_to_ast_string(*expr.right))
		}
	};
	ret
}
