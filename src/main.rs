pub mod ast_printer;
pub mod backend;
pub mod lexer;
pub mod parser;
pub mod representations;

//use clap::Parser;
//use std::collections::HashMap;

// use std::fs::File;
// use std::io::Write;
use crate::ast_printer::statement_pretty_printer;
use crate::backend::build;
use crate::lexer::lexer;
use crate::parser::parse_tokens;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

// #[derive(Parser, Debug)]
// struct Args {
//     #[arg(short = 'f')]
//     file_path: String,
// }

fn main() {
    //let args = Args::parse();

    let file_path = PathBuf::from("/home/aidan/PROJECTS/testcomp/test.ttc"); // for debug! change this
    let raw_code = std::fs::read_to_string(file_path).unwrap();

    let mut lexed_line: VecDeque<representations::Token> = lexer(raw_code);
    /*     println!("TOKENS: {:#?}\n\n", lexed_line) */

    let statements = parse_tokens(&mut lexed_line);

    println!("\n\n");

    statement_pretty_printer(&statements[0]);

    println!("\n\n");

    let (asm_lines, result_reg) = build(&statements);

    let output_file_path = PathBuf::from("test.asm");

    let mut output_string: String = r#"global _start

section .text

_start:
"#
    .to_string();

    for line in asm_lines {
        output_string.push_str("    ");
        output_string.push_str(line.as_str());
        output_string.push_str("\n");
    }

    output_string.push_str(format!(
        r#"    mov rax, 60
    mov rdi, {}
    syscall"#,
    result_reg).as_str());

    let mut output_file = File::create(output_file_path).expect("should work");
    output_file
        .write(output_string.as_bytes())
        .expect("shopulfd work");

    let _ = Command::new("make").status().unwrap();

    let compiled_status = Command::new("./test").status().unwrap();

    println!("\n\n{}\n", compiled_status);
}
