use std::collections::{HashMap, VecDeque};

use crate::representations::{Expression, Statement};

pub fn build(statements: &VecDeque<Statement>) -> Vec<String> {
    let mut symbol_register_hash = HashMap::<String, String>::new();
    let mut register_queue: VecDeque<&str> = VecDeque::from(["r8", "r9", "r10", "r11"]);
    let asm_lines = build_statement(&statements[0], &mut register_queue, &mut symbol_register_hash);
    asm_lines
}

fn build_statement(
    statement: &Statement,
    reg_queue: &mut VecDeque<&str>,
    symbol_hash: &mut HashMap<String, String>,
) -> Vec<String>{
    match statement {
        Statement::Assignment(_token, _id, expr) => {
            let (_, asm_lines) = build_expr(&expr, reg_queue, symbol_hash);
            for line in &asm_lines {
                println!("{}", line);
            }
            asm_lines
        }
    }
}

pub fn build_expr(
    expr: &Expression,
    reg_queue: &mut VecDeque<&str>,
    symbol_hash: &mut HashMap<String, String>,
) -> (String, Vec<String>) {
    let mut instruction_list: Vec<String> = Vec::new();
    match expr {
        Expression::Binary(left, op, right) => match left.as_ref() {
            Expression::Literal(left_token) => match right.as_ref() {
                Expression::Literal(right_token) => {
                    let reg = reg_queue.pop_front().expect("Should be a register here");
                    let instruction = format!(
                        "lea {}, [{} {} {}]",
                        reg,
                        left_token.lexeme(),
                        op.lexeme(),
                        right_token.lexeme()
                    );
                    instruction_list.push(instruction);
                    return (reg.to_string(), instruction_list);
                }
                Expression::Binary(_, _, _) => {
                    let (inner_reg, mut instructions) = build_expr(&right, reg_queue, symbol_hash);
                    instruction_list.append(&mut instructions);
                    instruction_list.push(format!(
                        "lea {}, [{} {} {}]",
                        inner_reg,
                        left_token.lexeme(),
                        op.lexeme(),
                        inner_reg
                    ));
                    return (inner_reg, instruction_list);
                },
                Expression::Group(_, inner_expr, _) => {
                    let (inner_reg, mut instructions) = build_expr(&inner_expr, reg_queue, symbol_hash);
                    instruction_list.append(&mut instructions);
                    instruction_list.push(format!(
                        "lea {}, [{} {} {}]",
                        inner_reg,
                        left_token.lexeme(),
                        op.lexeme(),
                        inner_reg
                    ));
                    return (inner_reg, instruction_list);
                },
 
                _ => panic!("Can't do this yet"),
            },
            _ => panic!("Can't do this yet"),
        },
        _ => panic!("Can't do this yet"),
    }
}
