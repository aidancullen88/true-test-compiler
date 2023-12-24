use crate::representations::{Block, Expression, Statement};

pub fn statement_pretty_printer(stmt: &Statement) {
    match stmt {
        Statement::Assignment(t, id, expr) => {
            print!("{} ({:#?}) = ", id, t);
            ast_pretty_printer(&expr);
        }
        Statement::If(expr, block) => {
            print!("if ");
            ast_pretty_printer(&expr);
            statement_pretty_printer(&block)
        }
        Statement::IfElse(expr, if_block, else_block) => {
            print!("if ");
            ast_pretty_printer(&expr);
            print!(" ");
            statement_pretty_printer(&if_block);
            print!("\nelse ");
            statement_pretty_printer(&else_block);
        }
        Statement::Block(block) => match block.as_ref() {
            Block::Statement(stmt) => statement_pretty_printer(&stmt),
            Block::Block(stmt, block) => {
                statement_pretty_printer(&stmt);
                block_pretty_printer(&block);
            }
        },
        Statement::While(expr, block) => {
            print!("while ");
            ast_pretty_printer(&expr);
            print!(" ");
            statement_pretty_printer(&block)
        }
        Statement::ReAssignment(id, expr) => {
            print!("{} = ", id);
            ast_pretty_printer(&expr);
        },
        Statement::Break => print!("break")
    }
}

pub fn block_pretty_printer(block: &Block) {
    match block {
        Block::Statement(stmt) => {
            print!(" {{\n    ");
            statement_pretty_printer(&stmt);
            print!("\n}}\n");
        }
        Block::Block(stmt, block) => {
            print!("{{\n    ");
            statement_pretty_printer(&stmt);
            block_pretty_printer(&block);
            print!("\n}}\n");
        }
    }
}

pub fn ast_pretty_printer(expr: &Expression) {
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
