pub mod ast_printer;
pub mod backend;
pub mod lexer;
pub mod parser;
pub mod representations;

use crate::ast_printer::statement_pretty_printer;
use crate::backend::build;
use crate::lexer::lexer;
use crate::parser::parse_tokens;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    let file_path = PathBuf::from("/home/aidan/PROJECTS/testcomp/test.ttc"); // for debug! change this
    let raw_code = std::fs::read_to_string(file_path).unwrap();

    let compiler_time = Instant::now();

    let mut lexed_line: std::collections::VecDeque<representations::Token> = lexer(raw_code);

    let mut symbol_table = HashMap::<String, representations::Symbol>::new();
    let mut statements = parse_tokens(&mut lexed_line, &mut symbol_table);

    println!("\n\n");
    for s in &statements {
        statement_pretty_printer(s);
        println!("\n")
    }

    let asm_lines = build(&mut statements, &mut symbol_table);

    println!("\n{:#?}\n", symbol_table);
    let comp_time = compiler_time.elapsed();

    let output_file_path = PathBuf::from("test.asm");
    let mut output_string: String = r#"global _start

section .text

_start:
"#
    .to_string();

    for line in asm_lines {
        if !line.contains(":") {
            output_string.push_str("    ");
        }
        output_string.push_str(line.as_str());
        output_string.push_str("\n");
    }

    output_string.push_str(
        format!(
            r#"    mov rax, 60
    syscall"#
        )
        .as_str(),
    );

    let mut output_file = File::create(output_file_path).expect("should work");
    output_file
        .write(output_string.as_bytes())
        .expect("shopulfd work");

    let _ = Command::new("make").status().unwrap();
    let compiled_status = Command::new("./test").status().unwrap();

    println!("{}\n", compiled_status);

    let elapsed = now.elapsed();

    println!("Total: {:.2?}", elapsed);
    println!("Comp only: {:.2?}", comp_time);
}
