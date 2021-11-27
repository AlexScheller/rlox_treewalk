use crate::parser;

pub fn expr_to_ast_string(expression: parser::Expr) -> String {
	let ret = match expression {
		parser::Expr::Binary(expr) => {
			format!(
				"({} {} {})",
				expr.operator.token,
				expr_to_ast_string(*expr.left),
				expr_to_ast_string(*expr.right)
			)
		}
		parser::Expr::Literal(string) => string,
	};
	ret
}
