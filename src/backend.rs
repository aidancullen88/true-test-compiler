use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::ast_printer::ast_pretty_printer;
use crate::representations::{Expression, Statement};

pub fn build(statements: &VecDeque<Statement>) -> Vec<String> {
    let mut symbol_register_hash = HashMap::<String, String>::new();
    let mut register_queue: VecDeque<String> =
        VecDeque::from(["r8", "r9", "r10", "r11"].map(String::from));
    let mut used_registers: VecDeque<String> = VecDeque::new();
    let asm_lines = build_statement(
        &statements[0],
        &mut register_queue,
        &mut used_registers,
        &mut symbol_register_hash,
    );
    asm_lines
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
            // if reg_queue.len() <= 1 {
            //     let reg1 = used_registers.pop_front().expect("If the regs have been used they should be in here");
            //     instruction_list.push(format!("push {}", &reg1));
            //     reg_queue.push_back(reg1);
            //     let reg2 = used_registers.pop_front().expect("Should be another reg here as well!");
            //     instruction_list.push(format!("push {}", &reg2));
            //     reg_queue.push_back(reg2);
            // }
            let right_reg = build_expr(
                right,
                instruction_list,
                reg_queue,
                used_registers,
                symbol_hash,
            );

            let left_reg = build_expr(
                left,
                instruction_list,
                reg_queue,
                used_registers,
                symbol_hash,
            );
            // if reg_queue.len() == 0 {
            //      let reg1 = used_registers.pop_front().expect("If the regs have been used they should be in here");
            //     instruction_list.push(format!("push {}", &reg1));
            //     reg_queue.push_back(reg1);
            // }
            match op.lexeme() {
                "+" => {
                    instruction_list.push(format!("add {}, {}", left_reg, right_reg));
                    println!("freed {}", &right_reg);
                    reg_queue.push_front(right_reg);
                }
                "*" => {
                    instruction_list.push(format!("mov rax, {}", left_reg));
                    instruction_list.push(format!("mul {}", right_reg));
                    instruction_list.push(format!("mov {}, rax", left_reg));
                    println!("freed {}", &right_reg);
                    reg_queue.push_front(right_reg);
                }
                _ => panic!("Can't handle {} yet", op.lexeme()),
            };
            left_reg
        }
        Expression::Literal(token) => {
            let reg = reg_queue
                .pop_front()
                .expect("Should be a register to put the literal into!");
            used_registers.push_back(reg.clone());
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
