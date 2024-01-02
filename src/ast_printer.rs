use crate::representations::{Assignment, Block, Expression, List, Literal, Statement, Type};

fn type_printer(ttp: &Type) {
    match ttp {
        Type::Bool | Type::Int | Type::None => print!("{:?}", ttp),
        Type::Pointer(inner_ttp) => {
            print!("*");
            type_printer(inner_ttp)
        }
        Type::Array(inner_ttp, size) => {
            type_printer(inner_ttp);
            print!("[{}]", size)
        }
    }
}

pub fn statement_pretty_printer(stmt: &Statement) {
    match stmt {
        Statement::Assignment(assign, expr) => {
            match assign {
                Assignment::Value(_type, symbol) | Assignment::Pointer(_type, symbol) => {
                    type_printer(_type);
                    print!(" {} = ", symbol);
                }
                Assignment::Mutation(symbol) => print!("{} = ", symbol),
            }
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
        Statement::Break => print!("break"),
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

fn list_pretty_printer(list: &List) {
    match list {
        List::Literal(literal) => literal_pretty_printer(literal),
        List::List(literal, list) => {
            literal_pretty_printer(literal);
            print!(", ");
            list_pretty_printer(list);
        }
    }
}

fn literal_pretty_printer(literal: &Literal) {
    match literal {
        Literal::Bool(token) | Literal::Int(token) | Literal::Symbol(token) => {
            print!("{}", token.lexeme())
        }
        Literal::List(list) => {
            print!("[");
            list_pretty_printer(list);
            print!("]");
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
        Expression::Literal(literal) => literal_pretty_printer(literal),
        Expression::Group(_, inner_expr, _) => {
            print!("group[");
            ast_pretty_printer(inner_expr);
            print!("]");
        }
    }
}
