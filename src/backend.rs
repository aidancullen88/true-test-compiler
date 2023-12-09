use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::representations::{Expression, Statement};

// Build a list of statements into their instructions: module entry point
pub fn build(statements: &VecDeque<Statement>) -> Vec<String> {
    // Not currently used, needed for indentifier addresses later
    let mut symbol_register_hash = HashMap::<String, String>::new();
    let asm_lines = build_statement(&statements[0], &mut symbol_register_hash);
    asm_lines
}

// Build one statement into asm instructions
fn build_statement(
    statement: &Statement,
    symbol_hash: &mut HashMap<String, String>,
) -> Vec<String> {
    // The registers free to do computations. Could add more
    let mut reg_list = VecDeque::<String>::from(["r8", "r9", "r10", "r11"].map(String::from));
    let mut instruction_list = Vec::<String>::new();
    match statement {
        Statement::Assignment(_token, _id, expr) => {
            let final_reg = build_expr(&expr, &mut reg_list, &mut instruction_list, symbol_hash);
            // Temporary check to make sure the rdi register is set to the computed value for exit
            match final_reg {
                InnerAddrType::Stack => instruction_list.push(format!("pop rdi")),
                InnerAddrType::Reg(reg) => instruction_list.push(format!("mov rdi, {}", reg)),
            }
            instruction_list
        }
    }
}

// Represent the possible places the return of a binary expr can be
enum InnerAddrType {
    Stack,
    Reg(String),
}

// Builds an expression into asm instructions
fn build_expr(
    expr: &Expression,
    reg_list: &mut VecDeque<String>,
    instruction_list: &mut Vec<String>,
    symbol_hash: &mut HashMap<String, String>,
) -> InnerAddrType {
    // Recursive match on the expression AST
    match expr {
        Expression::Binary(left, op, right) => {
            // Recurse into the tree
            let left_addr = build_expr(left, reg_list, instruction_list, symbol_hash);
            let right_addr = build_expr(right, reg_list, instruction_list, symbol_hash);

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
            }

            match left_addr {
                InnerAddrType::Stack => {
                    instruction_list.push(format!("pop rax"));
                    left_reg = "rax";
                }
                InnerAddrType::Reg(ref reg) => {
                    left_reg = &reg;
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
                InnerAddrType::Stack => match reg_list.pop_front() {
                    Some(reg) => {
                        instruction_list.push(format!("mov {}, rax", reg));
                        return InnerAddrType::Reg(reg);
                    }
                    None => {
                        instruction_list.push(format!("push rax"));
                        return InnerAddrType::Stack;
                    }
                },
            }
        }
        // When we get to a literal, the value is just pushed into a
        // reg or onto the stack
        Expression::Literal(token) => match reg_list.pop_front() {
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
        // A group just recurses straight away
        Expression::Group(_, expr, _) => build_expr(expr, reg_list, instruction_list, symbol_hash),
        _ => panic!("Expression must be a binary expression"),
    }
}
