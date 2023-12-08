use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::ast_printer::ast_pretty_printer;
use crate::representations::{Expression, Statement};

pub fn build(statements: &VecDeque<Statement>) -> (Vec<String>, String) {
    let mut symbol_register_hash = HashMap::<String, String>::new();
    let mut register_queue: VecDeque<String> =
        VecDeque::from(["r11", "r10", "r9", "r8"].map(String::from));
    let mut used_registers: VecDeque<String> = VecDeque::new();
    let asm_lines = build_statement(
        &statements[0],
        &mut register_queue,
        &mut used_registers,
        &mut symbol_register_hash,
    );
    (asm_lines, used_registers[0].clone())
}

fn build_statement(
    statement: &Statement,
    reg_queue: &mut VecDeque<String>,
    used_registers: &mut VecDeque<String>,
    symbol_hash: &mut HashMap<String, String>,
) -> Vec<String> {
    let mut instruction_list = Vec::<String>::new();
    match statement {
        Statement::Assignment(_token, _id, expr) => {
            let _ = build_expr(
                &expr,
                &mut instruction_list,
                reg_queue,
                used_registers,
                symbol_hash,
            );
            instruction_list
        }
    }
}

pub fn build_expr(
    expr: &Expression,
    instruction_list: &mut Vec<String>,
    reg_queue: &mut VecDeque<String>,
    used_registers: &mut VecDeque<String>,
    symbol_hash: &mut HashMap<String, String>,
) -> String {
    match expr {
        Expression::Binary(left, op, right) => {
            ast_pretty_printer(&expr);
            println!("\n");

            let left_addr = build_expr(
                left,
                instruction_list,
                reg_queue,
                used_registers,
                symbol_hash,
            );

             let right_addr = build_expr(
                right,
                instruction_list,
                reg_queue,
                used_registers,
                symbol_hash,
            );

            match op.lexeme() {
                "+" => {
                    instruction_list.push(format!("mov rax, {}", left_addr));
                    instruction_list.push(format!("mov rdx, {}", right_addr));
                    instruction_list.push(format!("add rax, rdx"));
                    instruction_list.push(format!("push rax"));
                    // println!("freed {}, {}", &left_reg, &right_reg);
                    // reg_queue.push_front(right_reg);
                    // used_registers.pop_front().expect("Should always be a used reg");
                }
                "*" => {
                    instruction_list.push(format!("mov rax, {}", left_addr));
                    instruction_list.push(format!("mov rdx, {}", right_addr));
                    instruction_list.push(format!("mul rdx"));
                    instruction_list.push(format!("push rax"));
                    // println!("freed {}", &right_reg);
                    // reg_queue.push_front(right_reg);
                    // used_registers.pop_front().expect("Should always be a used reg");
                }
                _ => panic!("Can't handle {} yet", op.lexeme()),
            };
            "rsp".to_string()
        }
        Expression::Literal(token) => {
            let reg = reg_queue
                .pop_front()
                .expect("Should be a register to put the literal into!");
            // used_registers.push_back(reg.clone());
            println!("{}, {}", &reg, &token.lexeme());
            instruction_list.push(format!("mov {}, {}", reg, token.lexeme()));
            reg
        }
        Expression::Group(_, expr, _) => {
            let inner_reg = build_expr(
                expr,
                instruction_list,
                reg_queue,
                used_registers,
                symbol_hash,
            );
            inner_reg
        }
        _ => panic!("Expression must be a binary expression"),
    }
}
