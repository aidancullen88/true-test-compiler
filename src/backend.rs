use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::representations::{Expression, InnerAddrType, Statement, Symbol, TokenType};

// Build a list of statements into their instructions: module entry point
pub fn build(
    statements: &mut VecDeque<Statement>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> Vec<String> {
    // The reg list is shared throughout the current function
    // It allows simple operations in expressions to be done
    // quickly without using the stack
    let mut reg_list = VecDeque::<String>::from(["r8", "r9", "r10", "r11"].map(String::from));
    // This is the final instruction list that the module
    // returns to the main function to be saved to the file
    let mut program_instruction_list = Vec::<String>::new();
    // Move the stack pointer into the base pointer so that we have a base point relative to each
    // variable that is saved in the function
    program_instruction_list.push(format!("mov rbp, rsp"));
    // This stack offset starts at 8 (for u64, currently the only data type supported)
    let mut stack_offset_counter = 8;
    while let Some(stmt_to_build) = statements.pop_front() {
        let mut instruction_list = build_statement(
            &stmt_to_build,
            &mut reg_list,
            symbol_table,
            &mut stack_offset_counter,
        );
        program_instruction_list.append(&mut instruction_list);
    }
    // FOR DEBUG ONLY: pop the last variable into rdi to act as the exit code for the generated asm
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
    // The instruction list for each statement. Appended above into the overall program instruction
    // list
    let mut instruction_list = Vec::<String>::new();
    match statement {
        // Type checking has already occured but we need the type info to save into our symbol
        // table once the expr is built
        Statement::Assignment(s_type, id, expr) => {
            let final_loc = build_expr(&expr, reg_list, &mut instruction_list, symbol_table);
            // Depending on what the expression computed was + how many statements have come before
            // this in the function, the location of a variable's value may be a register, the top
            // of the stack, or located at a stack offset (e.g. let int y = x;)
            match final_loc {
                // If the variable is on the stack it is guarenteed to be the top of the stack
                // after computing the expr. So, simply save that offset and increment by 8
                // Later, will need to increment by some size of type that could be saved within
                // the Type enum
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
                // If the value is in a register, push that onto the stack, save the current
                // offset, and free the register for further computations
                InnerAddrType::Reg(reg) => {
                    symbol_table.insert(
                        id.to_string(),
                        Symbol {
                            stack_offset: Some(*stack_offset_counter),
                            _type: s_type.clone(),
                        },
                    );
                    instruction_list.push(format!(
                        "push {}   ; save {} to [rbp - {}]",
                        reg, id, stack_offset_counter
                    ));
                    reg_list.push_back(reg);
                    *stack_offset_counter += 8;
                    instruction_list
                }
                // If it is a stack offset, then copy the value to the next place in the stack.
                // Currently there are no pointers/references implemented: everything is copied
                InnerAddrType::StackOffset(offset) => {
                    symbol_table.insert(
                        id.to_string(),
                        Symbol {
                            stack_offset: Some(*stack_offset_counter),
                            _type: s_type.clone(),
                        },
                    );
                    // Note that for u64s we move the entire qword at that stack address into rax.
                    // This would change for bool, char etc
                    instruction_list.push(format!("mov rax, qword [rbp - {}]", offset));
                    instruction_list.push(format!(
                        "push rax   ; save {} to [rbp - {}]",
                        id, stack_offset_counter
                    ));
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

            // Match the return addresses from the recursive build_expr calls
            // If the return address is on the stack then pop it and set the
            // side reg to rax/rcx.
            // If the return address is a stack offset, move that qword into the respective
            // register.
            // Otherwise, set the side reg to the reg that was returned
            let right_reg = match right_addr {
                InnerAddrType::Stack => {
                    instruction_list.push(format!("pop rcx"));
                    "rcx"
                }
                InnerAddrType::Reg(ref reg) => {
                    reg
                }
                InnerAddrType::StackOffset(offset) => {
                    instruction_list.push(format!("mov rcx, qword [rbp - {}]", offset));
                    "rcx"
                }
            };

            let left_reg = match left_addr {
                InnerAddrType::Stack => {
                    instruction_list.push(format!("pop rax"));
                    "rax"
                }
                InnerAddrType::Reg(ref reg) => {
                    reg
                }
                InnerAddrType::StackOffset(offset) => {
                    instruction_list.push(format!("mov rax, qword [rbp - {}]", offset));
                    "rax"
                }
            };

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
                    // div requires rdx (where it puts the remainder) to be 0 or it will segfault
                    instruction_list.push(format!("xor rdx, rdx"));
                    match left_reg {
                        // Same idea as mul above
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

            // If the right expr result was in a reg it can always be released.
            // If it was on the stack as a value it has been popped, if it was a relative address
            // to a variable value then it just stays there
            match right_addr {
                InnerAddrType::Reg(reg) => reg_list.push_back(reg),
                _ => (),
            }

            // If the left value was a reg, we can store the result in there
            // If not, we can check for free regs and store the result in there
            // If not, we can push it onto the stack
            // This logic is the same if it was originally a stack offset
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
        // If the primary token is an id, we get it's address
        Expression::Literal(token) => match token.token_type() {
            // match to see if the symbol already exists: if not it's an error
            TokenType::Identifier => match symbol_table.get(token.lexeme()) {
                // Match to see if the symbol is initialised. If not, it's an error
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
