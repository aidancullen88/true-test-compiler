use core::panic;
use std::collections::{HashMap, VecDeque};

use crate::representations::{
    Block, Expression, InnerAddrType, Statement, Symbol, TokenType, Type,
};

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
    let mut label_counter = 1;

    while let Some(stmt_to_build) = statements.pop_front() {
        let mut instruction_list = build_statement(
            &stmt_to_build,
            &mut reg_list,
            symbol_table,
            &mut stack_offset_counter,
            &mut label_counter,
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
    label_counter: &mut u32,
) -> Vec<String> {
    // The instruction list for each statement. Appended above into the overall program instruction
    // list
    let mut instruction_list = Vec::<String>::new();
    match statement {
        // Type checking has already occured but we need the type info to save into our symbol
        // table once the expr is built
        Statement::Assignment(_, id, expr) => {
            let final_loc = build_expr(&expr, reg_list, &mut instruction_list, symbol_table);
            let symbol_info = symbol_table
                .get_mut(id)
                .expect("Symbol should always already exist in table");
            // Depending on what the expression computed was + how many statements have come before
            // this in the function, the location of a variable's value may be a register, the top
            // of the stack, or located at a stack offset (e.g. let int y = x;)
            match final_loc {
                // If the variable is on the stack it is guarenteed to be the top of the stack
                // after computing the expr. So, simply save that offset and increment by 8
                // Later, will need to increment by some size of type that could be saved within
                // the Type enum
                InnerAddrType::Stack => {
                    symbol_info.stack_offset = Some(*stack_offset_counter);
                    *stack_offset_counter += 8;
                    instruction_list
                }
                // If the value is in a register, push that onto the stack, save the current
                // offset, and free the register for further computations
                InnerAddrType::Reg(reg) => {
                    symbol_info.stack_offset = Some(*stack_offset_counter);
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
                    symbol_info.stack_offset = Some(*stack_offset_counter);
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
        Statement::ReAssignment(id, expr) => {
            let final_loc = build_expr(&expr, reg_list, &mut instruction_list, symbol_table);
            let symbol_info = symbol_table
                .get(id)
                .expect("id should already be in symbol table");
            if !symbol_info.mutable {
                panic!("Double check for mutable reassignment: shouldn't happen!")
            }
            match final_loc {
                InnerAddrType::Stack => {
                    instruction_list.push(format!("pop rax"));
                    instruction_list.push(format!(
                        "mov qword [rbp - {}], rax",
                        symbol_info.stack_offset.expect("Should be offset")
                    ));
                    instruction_list
                }
                InnerAddrType::Reg(reg) => {
                    instruction_list.push(format!(
                        "mov qword [rbp - {}], {}",
                        symbol_info.stack_offset.expect("should be offset"),
                        reg
                    ));
                    instruction_list
                }
                InnerAddrType::StackOffset(offset) => {
                    instruction_list.push(format!("mov rax, qword [rbp - {}]", offset));
                    instruction_list.push(format!(
                        "mov qword [rbp - {}], rax",
                        symbol_info.stack_offset.expect("Should be offset")
                    ));
                    instruction_list
                }
            }
        }
        Statement::If(expr, if_block) => {
            let if_label = format!("if_{}", label_counter);
            build_cmp_instructions(
                expr,
                &if_label,
                &mut instruction_list,
                symbol_table,
                reg_list,
            );
            instruction_list.push(format!("jmp end_{}", if_label));
            instruction_list.push(format!("{}:", if_label));
            instruction_list.append(&mut build_statement(
                &if_block,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            ));
            instruction_list.push(format!("end_{}:", if_label));
            *label_counter += 1;
            instruction_list
        }
        Statement::IfElse(expr, if_block, else_block) => {
            let if_else_label = format!("if_else_{}", label_counter);
            build_cmp_instructions(
                expr,
                &if_else_label,
                &mut instruction_list,
                symbol_table,
                reg_list,
            );
            instruction_list.append(&mut build_statement(
                &else_block,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            ));
            instruction_list.push(format!("jmp end_{}", if_else_label));
            instruction_list.push(format!("{}:", if_else_label));
            instruction_list.append(&mut build_statement(
                &if_block,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            ));
            instruction_list.push(format!("end_{}:", if_else_label));
            *label_counter += 1;
            instruction_list
        }
        Statement::Block(block) => {
            instruction_list.append(&mut build_block(
                block,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            ));
            instruction_list
        }
        Statement::While(expr, while_block) => {
            let while_label = format!("while_{}", label_counter);
            instruction_list.push(format!("start_{}:", while_label));
            let end_while_label = format!("end_while_{}", label_counter);
            build_cmp_instructions(
                expr,
                &while_label,
                &mut instruction_list,
                symbol_table,
                reg_list,
            );
            instruction_list.push(format!("jmp {}", end_while_label));
            instruction_list.push(format!("{}:", while_label));
            instruction_list.append(&mut build_statement(
                &while_block,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            ));
            instruction_list.push(format!("jmp start_{}", while_label));
            instruction_list.push(format!("{}:", end_while_label));
            *label_counter += 1;
            instruction_list
        }
    }
}

fn build_cmp_instructions(
    expr: &Expression,
    if_label: &str,
    instruction_list: &mut Vec<String>,
    symbol_table: &mut HashMap<String, Symbol>,
    reg_list: &mut VecDeque<String>,
) {
    match expr {
        Expression::Binary(left_expr, op, right_expr) => {
            // compute the value of each expr and move it into _addr
            // Then, get the string representation of left_addr
            let left_addr = get_inner_register(
                &build_expr(&left_expr, reg_list, instruction_list, symbol_table),
                "rax",
                instruction_list,
            );
            let right_addr = get_inner_register(
                &build_expr(&right_expr, reg_list, instruction_list, symbol_table),
                "rcx",
                instruction_list,
            );
            // Add the cmp instruction using the addresses computed above
            instruction_list.push(format!("cmp {}, {}", left_addr, right_addr));
            // Match the operation and get the instruction
            let jump_instr = match op.lexeme() {
                "==" => "je",
                "!=" => "jne",
                "<" => "jl",
                ">" => "jg",
                "<=" => "jle",
                ">=" => "jge",
                _ => panic!("Should never be a non bool op in if statement expr"),
            };
            // Add the jump instruction for the end of the if block, along with some nice debug
            // comments
            instruction_list.push(format!(
                "{} {}\n    ; else block",
                jump_instr,
                if_label,
            ));
        }
        _ => panic!("can;t do this yet"),
    }
}

fn build_block(
    block: &Block,
    reg_list: &mut VecDeque<String>,
    symbol_table: &mut HashMap<String, Symbol>,
    stack_offset_counter: &mut u32,
    label_counter: &mut u32,
) -> Vec<String> {
    match block {
        Block::Statement(stmt) => {
            return build_statement(
                &stmt,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            )
        }
        Block::Block(stmt, block) => {
            let mut stmt_instructions = build_statement(
                &stmt,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            );
            let mut block_instructions = build_block(
                block,
                reg_list,
                symbol_table,
                stack_offset_counter,
                label_counter,
            );
            stmt_instructions.append(&mut block_instructions);
            stmt_instructions
        }
    }
}

fn get_inner_register(
    addr: &InnerAddrType,
    default: &str,
    instruction_list: &mut Vec<String>,
) -> String {
    // Match the return addresses from the recursive build_expr calls
    // If the return address is on the stack then pop it and set the
    // side reg to rax/rcx.
    // If the return address is a stack offset, move that qword into the respective
    // register.
    // Otherwise, set the side reg to the reg that was returned
    match addr {
        InnerAddrType::Stack => {
            instruction_list.push(format!("pop {}", default));
            default.to_string()
        }
        InnerAddrType::Reg(ref reg) => reg.to_string(),
        InnerAddrType::StackOffset(offset) => {
            instruction_list.push(format!("mov {}, qword [rbp - {}]", default, offset));
            default.to_string()
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

            let right_reg = get_inner_register(&right_addr, "rcx", instruction_list);
            let left_reg = get_inner_register(&left_addr, "rax", instruction_list);
            // Add the actual operation instructions
            match op.lexeme() {
                "+" => {
                    instruction_list.push(format!("add {}, {}", left_reg, right_reg));
                }
                "-" => {
                    instruction_list.push(format!("sub {}, {}", left_reg, right_reg));
                }
                operation @ ("*" | "/") => {
                    instruction_list.append(&mut build_factor_op(&left_reg, &right_reg, operation))
                }
                operation @ ("==" | "!=" | "<" | ">" | "<=" | ">=") => instruction_list
                    .append(&mut build_comparison_op(&left_reg, &right_reg, operation)),
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
                    match token._type() {
                        Type::Bool => {
                            let bool_int = bool_to_int(token.lexeme());
                            instruction_list.push(format!("mov {}b, {}", reg, bool_int));
                        }
                        Type::Int => {
                            instruction_list.push(format!("mov {}, {}", reg, token.lexeme()))
                        }
                        _ => panic!(
                            "Literal of type none at {}:{}",
                            token.line_number(),
                            token.line_index(),
                        ),
                    }
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

fn bool_to_int(bool: &str) -> u8 {
    match bool {
        "true" => 1,
        "false" => 0,
        _ => panic!("{} is not a bool value!", bool),
    }
}

fn build_comparison_op(left_reg: &str, right_reg: &str, operation: &str) -> Vec<String> {
    let compare = format!("cmp {}, {}", left_reg, right_reg);
    let set8 = match operation {
        "==" => format!("sete al"),
        "<" => format!("setb al"),
        ">" => format!("seta al"),
        "!=" => format!("setne al"),
        "<=" => format!("setbe al"),
        ">=" => format!("setae al"),
        _ => panic!("Unrecognised op {}!", operation),
    };
    let assign = format!("movzx {}, al", left_reg);
    vec![compare, set8, assign]
}

fn build_factor_op(left_reg: &str, right_reg: &str, operation: &str) -> Vec<String> {
    let mut factor_op = Vec::<String>::new();
    factor_op.push(format!("mov rax, {}", left_reg));
    let mut op = match operation {
        "*" => vec![format!("mul {}", right_reg)],
        "/" => vec![format!("xor rdx, rdx"), format!("div {}", right_reg)],
        _ => panic!("Unrecognised factor op {}", operation),
    };
    if left_reg == "rax" {
        return op;
    };
    factor_op.append(&mut op);
    factor_op.push(format!("mov {}, rax", left_reg));
    factor_op
}
