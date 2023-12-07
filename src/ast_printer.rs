use crate::representations::{Expression, Statement};

pub fn statement_pretty_printer(stmt: &Statement) {
    match stmt {
        Statement::Assignment(t, id, expr) => {
            print!("{} ({:#?}) = ", id, t);
            ast_pretty_printer(&expr);
        }
    }
}

fn ast_pretty_printer(expr: &Expression) {
    match expr {
        Expression::Binary(left, op, right) => {
            print!("(");
            //print!("{} ", op.lexeme());
            ast_pretty_printer(left);
            print!(" {} ", op.lexeme());
            ast_pretty_printer(right);
            //print!(" {}", op.lexeme());
            print!(")");
        }
        Expression::Unary(op, right) => {
            print!("{}", op.lexeme());
            ast_pretty_printer(right);
        }
        Expression::Literal(token) => {
            print!("{}", &token.lexeme());
        }
        Expression::Group(_, inner_expr, _) => {
            print!("group[");
            ast_pretty_printer(inner_expr);
            print!("]");
        }
    }
}
