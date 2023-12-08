use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::representations::{Expression, Statement};

pub fn build(statements: &VecDeque<Statement>) -> Vec<String> {
    let mut symbol_register_hash = HashMap::<String, String>::new();
    let asm_lines = build_statement(
        &statements[0],
        &mut symbol_register_hash,
    );
    asm_lines
}

fn build_statement(
    statement: &Statement,
    symbol_hash: &mut HashMap<String, String>,
) -> Vec<String> {
    let mut instruction_list = Vec::<String>::new();
    match statement {
        Statement::Assignment(_token, _id, expr) => {
            build_expr(
                &expr,
                &mut instruction_list,
                symbol_hash,
            );
            instruction_list
        }
    }
}

pub fn build_expr(
    expr: &Expression,
    instruction_list: &mut Vec<String>,
    symbol_hash: &mut HashMap<String, String>,
) -> () {
    match expr {
        Expression::Binary(left, op, right) => {
            build_expr(
                left,
                instruction_list,
                symbol_hash,
            );

            build_expr(
                right,
                instruction_list,
                symbol_hash,
            );

            match op.lexeme() {
                "+" => {
                    instruction_list.push(format!("pop rdx"));
                    instruction_list.push(format!("pop rax"));
                    instruction_list.push(format!("add rax, rdx"));
                    instruction_list.push(format!("push rax"));
                }
                "*" => {
                    instruction_list.push(format!("pop rdx"));
                    instruction_list.push(format!("pop rax"));
                    instruction_list.push(format!("mul rdx"));
                    instruction_list.push(format!("push rax"));
                }
                _ => panic!("Can't handle {} yet", op.lexeme()),
            };
        }
        Expression::Literal(token) => {
            instruction_list.push(format!("mov rax, {}", token.lexeme()));
            instruction_list.push(format!("push rax"));
        }
        Expression::Group(_, expr, _) => {
            build_expr(
                expr,
                instruction_list,
                symbol_hash,
            );
        }
        _ => panic!("Expression must be a binary expression"),
    }
}
