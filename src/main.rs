//use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

// #[derive(Parser, Debug)]
// struct Args {
//     #[arg(short = 'f')]
//     file_path: String,
// }

fn main() {
    //let args = Args::parse();

    let file_path = PathBuf::from("/home/aidan/PROJECTS/testcomp/test.ttc"); // for debug! change this
    let raw_code = std::fs::read_to_string(file_path).unwrap();

    let lexed_line = lexer(raw_code);
    println!("{:#?}", lexed_line);

    // let mut asm_file = File::create("test.asm").unwrap();

    // asm_file
    //     .write_all(build_asm(lexed_line).as_bytes())
    //     .unwrap();
}

#[derive(Debug, Clone)]
enum TokenType {
    Exit,
    IntLit,
    Semi,
    Invalid,
    LeftParen,
    RightParen,
    Identifier,
    Assign,
    IsEqual,
}

#[derive(Debug)]
struct Token {
    token: TokenType,
    value: Option<String>,
    lexeme: String,
    line_number: usize,
    line_index: usize,
}

fn lexer(mut src: String) -> Vec<Token> {
    let single_char_keys = HashMap::from([
        (';', TokenType::Semi),
        ('(', TokenType::LeftParen),
        (')', TokenType::RightParen),
    ]);
    let mut tokens = Vec::<Token>::new();
    // src_index keeps the total position in the src
    let mut src_index = 0;
    let src_length = src.len();
    let mut line_number = 0;
    let mut line_index = 1;

    // as we consume the src, src_index approachs the original src_length!
    while src_index != src_length {
        let first_char: char;

        // Break the loop if there's no char in src
        // This should never be hit as we check the length
        if let Some(c) = src.chars().nth(0) {
            first_char = c;
        } else {
            break;
        };

        // Match the first character in the remaining src.
        // On simple (1-char) match:
        //      Consume the matched char and produce a token, src_index += 1
        // On complex (multi-char) match:
        //      Look ahead and consume all chars, produce a token and the no.
        //      of chars consumed
        match single_char_keys.get(&first_char) {
            Some(token_type) => {
                tokens.push(consume_token(
                    &mut src,
                    1,
                    token_type.clone(),
                    None,
                    line_number,
                    line_index,
                ));
                src_index += 1;
                line_index += 1
            }
            // If the first char isn't an single char lexeme
            None => match first_char {
                ' ' => {
                    src.drain(..1);
                    src_index += 1;
                    line_index += 1;
                }
                '\n' => {
                    src.drain(..1);
                    src_index += 1;
                    line_index = 1;
                    line_number += 1;
                }
                '=' => {
                    let (token, index) =
                        check_multi_char_operator(&mut src, line_number, line_index, '=');
                    tokens.push(token);
                    src_index += index;
                    line_index += index;
                }
                _ => {
                    let (token, index) =
                        check_literal_identifier_or_keyword(&mut src, line_number, line_index);
                    tokens.push(token);
                    src_index += index;
                    line_index += index;
                }
            },
        }
    }
    tokens
}

fn consume_token(
    src: &mut String,
    chars_to_consume: usize,
    token_type: TokenType,
    value: Option<String>,
    line_number: usize,
    line_index: usize,
) -> Token {
    // remove the amount of characters specified from the string
    let lexeme = src.drain(..chars_to_consume);
    println!("lexeme is {}", &lexeme.as_str());
    // Return the token created
    Token {
        token: token_type.clone(),
        value,
        lexeme: lexeme.as_str().to_string(),
        line_number,
        line_index,
    }
}

fn check_literal_identifier_or_keyword(
    src: &mut String,
    line_number: usize,
    line_index: usize,
) -> (Token, usize) {
    // Could find a better way to store this Hashmap but w/e
    let lexume_keys: HashMap<&str, TokenType> = HashMap::from([("exit", TokenType::Exit)]);
    let lexume_index: usize;
    //let mut broke = false;
    let lexeme: &str;

    match src.find(|c: char| -> bool { !is_valid_identfier_char(&c) }) {
        Some(i) => {
            lexeme = &src[..i];
            lexume_index = i
        }
        None => {
            lexeme = &src;
            lexume_index = src.len();
        }
    };

    // check if the lexeme is a literal (starts with a number)
    match lexeme
        .chars()
        .nth(0)
        .expect("lexeme should always have at least 1 char!")
    {
        c if c.is_ascii_digit() => {
            return (
                consume_token(
                    src,
                    lexume_index,
                    TokenType::IntLit,
                    Some(lexeme.to_string()),
                    line_number,
                    line_index,
                ),
                lexume_index,
            );
        }
        _ =>
        // if it's not a literal must be a id or kw, check the hashmap!
        {
            match lexume_keys.get(&src[..lexume_index]) {
                Some(token_type) => (
                    consume_token(
                        src,
                        lexume_index,
                        token_type.clone(),
                        None,
                        line_number,
                        line_index,
                    ),
                    lexume_index,
                ),
                None => (
                    consume_token(
                        src,
                        lexume_index,
                        TokenType::Identifier,
                        None,
                        line_number,
                        line_index,
                    ),
                    lexume_index,
                ),
            }
        }
    }
}

fn is_valid_identfier_char(c: &char) -> bool {
    if c.is_alphanumeric() || c == &'_' {
        return true;
    } else {
        return false;
    }
}

fn check_multi_char_operator(
    src: &mut String,
    line_number: usize,
    line_index: usize,
    to_match: char,
) -> (Token, usize) {
    let two_char_ops = HashMap::from([("==", TokenType::IsEqual)]);
    let one_char_ops = HashMap::from([('=', TokenType::Assign)]);
    // Match the next char in src
    match look_ahead(src, to_match) {
        // lookup the two char op table to get TokenType
        true => match two_char_ops.get(&src[..2]) {
            Some(token_type) => (
                consume_token(src, 2, token_type.clone(), None, line_number, line_index),
                2,
            ),
            // If none, add invalid (for now)
            None => (
                consume_token(src, 2, TokenType::Invalid, None, line_number, line_index),
                2,
            ),
        },
        // The next char didn't match OR there was no next char in src
        // Lookup in the one char ops table to get TokenType
        false => match one_char_ops.get(
            &src.chars()
                .nth(0)
                .expect("Should always be at least 1 char in src"),
        ) {
            Some(token_type) => (
                consume_token(src, 1, token_type.clone(), None, line_number, line_index),
                1,
            ),
            // If none, add invalid (for now)
            None => (
                consume_token(src, 1, TokenType::Invalid, None, line_number, line_index),
                1,
            ),
        },
    }
}

fn look_ahead(src: &str, to_match: char) -> bool {
    // Look at the next character in src and check if it matches
    match src.chars().nth(1) {
        Some(c) => {
            if c == to_match {
                return true;
            } else {
                return false;
            }
        }
        // If there's nothing after the first char, it must be
        // a one-char op
        None => return false,
    }
}

// This is temporary as in future will build from parse tree
fn build_asm(mut token_list: Vec<Token>) -> String {
    let mut asm_code = String::new();
    while token_list.len() != 0 {
        match token_list[0].token {
            // make this whole hierarchy better
            TokenType::Exit => {
                if token_list.len() < 3 {
                    asm_code.push_str("not enough tokens idiot!");
                    break;
                }
                token_list.remove(0); // move removes to end
                let should_be_semi = token_list.remove(1);
                match should_be_semi.token {
                    TokenType::Semi => asm_code.push_str(
                        format_return(&token_list.remove(0).value.clone().expect("value")).as_str(),
                    ),
                    _ => asm_code.push_str("Should be a semicolon!"),
                }
            }

            _ => asm_code.push_str("Unrecognised Syntax!"),
        }
    }
    asm_code
}

fn format_return(return_value: &str) -> String {
    format!(
        r#"global _start

section .text

_start:
    mov rax, 60
    mov rdi, {return_value}
    syscall"#
    )
}

// fn lex_code(raw_code: String) -> Vec<Token> {
//     let mut token_vec = Vec::<Token>::new();
//     let mut char_buffer = String::new();
//     for c in raw_code.chars() {
//         if c.is_alphanumeric() {
//             char_buffer.push(c);
//         } else {
//             // if there's characters in the buffer (i.e. not two whitespace in a row)
//             // then match the buffer to a token and save the token to token_vec
//             if char_buffer.len() != 0 {
//                 let first_char = char_buffer.as_bytes()[0];
//                 if first_char.is_ascii_alphabetic() {
//                     match char_buffer.as_str() {
//                         "exit" => token_vec.push(Token {
//                             token: TokenType::Exit,
//                             value: None,
//                         }),
//                         _ => token_vec.push(Token {
//                             token: TokenType::Invalid,
//                             value: None,
//                         }),
//                     }
//                 } else if first_char.is_ascii_digit() {
//                     token_vec.push(Token {
//                         token: TokenType::IntLit,
//                         value: Some(char_buffer),
//                     })
//                 }
//                 // If we're currently at the end of a statement, we want to save the
//                 // ';' as well, so check for that!
//                 if c == ';' {
//                     token_vec.push(Token {
//                         token: TokenType::Semi,
//                         value: None,
//                     })
//                 }
//                 // clear the buffer to read in the next character
//                 char_buffer.clear();
//             // If the char buffer is empty that means 2 non alpha-num characters
//             // in a row. This would be fine for ws but not for ; :(
//             } else {
//                 continue;
//             }
//         }
//     }
//     token_vec
// }

// fn consume_raw_code(raw_code: String) -> (Option<String>, String) {
//     let mut char_buffer = String::new();
//     for (i, c) in raw_code.chars().enumerate() {
//         if c.is_alphanumeric() {
//             char_buffer.push(c);
//         } else if c.is_ascii_whitespace() {
//             match char_buffer.as_str() {
//                 "ret" => return (Some(char_buffer), raw_code),
//                 _ => return (None, raw_code),
//             }
//         if
//     }
//     (None, "".to_string())
// }}
