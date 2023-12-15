use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::representations::{Expression, InnerAddrType, Statement, Symbol, TokenType};

// Build a list of statements into their instructions: module entry point
pub fn build(
    statements: &mut VecDeque<Statement>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> Vec<String> {
    let mut reg_list = VecDeque::<String>::from(["r8", "r9", "r10", "r11"].map(String::from));
    let mut program_instruction_list = Vec::<String>::new();
    program_instruction_list.push(format!("mov rbp, rsp"));
    let mut stack_offset_counter = 8;
    while statements.len() != 0 {
        let stmt_to_build = statements
            .pop_front()
            .expect("We check for an element in the while loop");
        let mut instruction_list = build_statement(
            &stmt_to_build,
            &mut reg_list,
            symbol_table,
            &mut stack_offset_counter,
        );
        program_instruction_list.append(&mut instruction_list);
    }
    program_instruction_list.push(format!("pop rdi"));
    program_instruction_list
}

// Build one statement into asm instructions
fn build_statement(
    statement: &Statement,
    reg_list: &mut VecDeque<String>,
    symbol_table: &mut HashMap<String, Symbol>,
    stack_offset_counter: &mut u32,
) -> Vec<String> {
    // The registers free to do computations. Could add more
    let mut instruction_list = Vec::<String>::new();
    match statement {
        Statement::Assignment(s_type, id, expr) => {
            let final_loc = build_expr(&expr, reg_list, &mut instruction_list, symbol_table);
            // Temporary check to make sure the rdi register is set to the computed value for exit
            match final_loc {
                InnerAddrType::Stack => {
                    symbol_table.insert(
                        id.to_string(),
                        Symbol {
                            stack_offset: Some(*stack_offset_counter),
                            _type: s_type.clone(),
                        },
                    );
                    *stack_offset_counter += 8;
                    instruction_list
                }
                InnerAddrType::Reg(reg) => {
                    symbol_table.insert(
                        id.to_string(),
                        Symbol {
                            stack_offset: Some(*stack_offset_counter),
                            _type: s_type.clone(),
                        },
                    );
                    instruction_list.push(format!("push {}   ; save {} to [rbp - {}]", reg, id, stack_offset_counter));
                    *stack_offset_counter += 8;
                    instruction_list
                }
                InnerAddrType::StackOffset(offset) => {
                    symbol_table.insert(
                        id.to_string(),
                        Symbol {
                            stack_offset: Some(*stack_offset_counter),
                            _type: s_type.clone(),
                        },
                    );
                    instruction_list.push(format!("mov rax, qword [rbp - {}]", offset));
                    instruction_list.push(format!("push rax   ; save {} to [rbp - {}]", id, stack_offset_counter));
                    *stack_offset_counter += 8;
                    instruction_list
                }
            }
        }
    }
}

// Builds an expression into asm instructions
fn build_expr(
    expr: &Expression,
    reg_list: &mut VecDeque<String>,
    instruction_list: &mut Vec<String>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> InnerAddrType {
    // Recursive match on the expression AST
    match expr {
        Expression::Binary(left, op, right) => {
            // Recurse into the tree
            let left_addr = build_expr(left, reg_list, instruction_list, symbol_table);
            let right_addr = build_expr(right, reg_list, instruction_list, symbol_table);

            // Init left and right reg to be assigned in the match
            let left_reg: &str;
            let right_reg: &str;

            // Match the return addresses from the recursive build_expr calls
            // If the return address is on the stack then pop it and set the
            // side reg to rax/rcx
            // Otherwise, set the side reg to the reg that was returned
            match right_addr {
                InnerAddrType::Stack => {
                    instruction_list.push(format!("pop rcx"));
                    right_reg = "rcx";
                }
                InnerAddrType::Reg(ref reg) => {
                    right_reg = reg;
                }
                InnerAddrType::StackOffset(offset) => {
                    instruction_list.push(format!("mov rcx, qword [rbp - {}]", offset));
                    right_reg = "rcx";
                }
            }

            match left_addr {
                InnerAddrType::Stack => {
                    instruction_list.push(format!("pop rax"));
                    left_reg = "rax";
                }
                InnerAddrType::Reg(ref reg) => {
                    left_reg = &reg;
                }
                InnerAddrType::StackOffset(offset) => {
                    instruction_list.push(format!("mov rax, qword [rbp - {}]", offset));
                    left_reg = "rax";
                }
            }

            // Add the actual operation instructions
            match op.lexeme() {
                "+" => {
                    instruction_list.push(format!("add {}, {}", left_reg, right_reg));
                }
                "*" => match left_reg {
                    // match on left reg to see if our value is in rax already
                    // Mul requires the left operand in rax, so if it is
                    // then just use that: otherwise move left_reg into rax
                    // and then back out once the op is done
                    "rax" => instruction_list.push(format!("mul {}", right_reg)),
                    _ => {
                        instruction_list.push(format!("mov rax, {}", left_reg));
                        instruction_list.push(format!("mul {}", right_reg));
                        instruction_list.push(format!("mov {}, rax", left_reg));
                    }
                },
                "-" => {
                    instruction_list.push(format!("sub {}, {}", left_reg, right_reg));
                }
                "/" => {
                    instruction_list.push(format!("xor rdx, rdx"));
                    match left_reg {
                        "rax" => instruction_list.push(format!("div {}", right_reg)),
                        _ => {
                            instruction_list.push(format!("mov rax, {}", left_reg));
                            instruction_list.push(format!("div {}", right_reg));
                            instruction_list.push(format!("mov {}, rax", left_reg));
                        }
                    }
                }
                // Other types of op that aren't implemented yet like ^ etc
                _ => panic!("Can't handle {} yet", op.lexeme()),
            };

            // If the right expr result was in a reg it can always be released
            match right_addr {
                InnerAddrType::Reg(reg) => reg_list.push_back(reg),
                _ => (),
            }

            // If the left value was a reg, we can store the result in there
            // If not, we can check for free regs and store the result in there
            // If not, we can push it onto the stack
            match left_addr {
                InnerAddrType::Reg(_) => {
                    return InnerAddrType::Reg(left_reg.to_string());
                }
                InnerAddrType::Stack | InnerAddrType::StackOffset(_) => {
                    match reg_list.pop_front() {
                        Some(reg) => {
                            instruction_list.push(format!("mov {}, rax", reg));
                            return InnerAddrType::Reg(reg);
                        }
                        None => {
                            instruction_list.push(format!("push rax"));
                            return InnerAddrType::Stack;
                        }
                    }
                }
            }
        }
        // When we get to a literal, the value is just pushed into a
        // reg or onto the stack
        Expression::Literal(token) => match token.token_type() {
            TokenType::Identifier => match symbol_table.get(token.lexeme()) {
                Some(symbol_info) => match symbol_info.stack_offset {
                    Some(offset) => {
                        return InnerAddrType::StackOffset(offset);
                    }
                    None => panic!("{} is referenced before initialisation", token.lexeme()),
                },
                None => panic!("{} is referenced before declaration", token.lexeme()),
            },
            TokenType::Literal => match reg_list.pop_front() {
                Some(reg) => {
                    instruction_list.push(format!("mov {}, {}", reg, token.lexeme()));
                    return InnerAddrType::Reg(reg);
                }
                None => {
                    instruction_list.push(format!("mov rax, {}", token.lexeme()));
                    instruction_list.push(format!("push rax"));
                    return InnerAddrType::Stack;
                }
            },
            _ => panic!("token is not a literal or id {}", token.lexeme()),
        },
        // A group just recurses straight away
        Expression::Group(_, expr, _) => build_expr(expr, reg_list, instruction_list, symbol_table),
        _ => panic!("Expression must be a binary expression"),
    }
}
