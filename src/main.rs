use core::panic;
//use clap::Parser;
//use std::collections::HashMap;
use std::collections::VecDeque;
// use std::fs::File;
// use std::io::Write;
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

    let mut lexed_line = lexer(raw_code);
    println!("TOKENS: {:#?}\n\n", lexed_line);

    let expressions = parse_tokens(&mut lexed_line);

    println!("\n");

    //ast_pretty_printer(&expressions[0]);

    println!("\n");

    // let mut asm_file = File::create("test.asm").unwrap();

    // asm_file
    //     .write_all(build_asm(lexed_line).as_bytes())
    //     .unwrap();
}

#[derive(Debug, Clone)]
struct Token {
    _type: Type,
    lexeme: String,
    line_number: usize,
    line_index: usize,
}

#[derive(Debug, Clone)]
enum Type {
    Int,
    Bool,
    None,
}

fn lexer(mut src: String) -> VecDeque<Token> {
    let mut tokens = VecDeque::<Token>::new();
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
        match first_char {
            ';' | '(' | ')' | '/' | '*' | '-' | '+' | '<' | '>' => {
                tokens.push_back(consume_token(
                    &mut src,
                    1,
                    Type::None,
                    line_number,
                    line_index,
                ));
                src_index += 1;
                line_index += 1
            }
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
                tokens.push_back(token);
                src_index += index;
                line_index += index;
            }
            _ => {
                let (token, index) =
                    check_literal_identifier_or_keyword(&mut src, line_number, line_index);
                tokens.push_back(token);
                src_index += index;
                line_index += index;
            }
        }
    }
    tokens
}

fn consume_token(
    src: &mut String,
    chars_to_consume: usize,
    _type: Type,
    line_number: usize,
    line_index: usize,
) -> Token {
    // remove the amount of characters specified from the string
    let lexeme = src.drain(..chars_to_consume);
    println!("lexeme is {}", &lexeme.as_str());
    // Return the token created
    Token {
        _type,
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
    //let lexume_keys: HashMap<&str, TokenType> = HashMap::from([("exit", TokenType::Exit)]);
    // let literal_keys: HashMap<&str, TokenType> = HashMap::from([
    //     ("true", TokenType::True),
    //     ("false", TokenType::False),
    // ]);
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
                consume_token(src, lexume_index, Type::Int, line_number, line_index),
                lexume_index,
            );
        }
        _ => match lexeme {
            "true" => (
                consume_token(src, lexume_index, Type::Bool, line_number, line_index),
                lexume_index,
            ),
            "false" => (
                consume_token(src, lexume_index, Type::Bool, line_number, line_index),
                lexume_index,
            ),
            "exit" | "let" | "int" => (
                consume_token(src, lexume_index, Type::None, line_number, line_index),
                lexume_index,
            ),
            _ => (
                consume_token(src, lexume_index, Type::None, line_number, line_index),
                lexume_index,
            ),
        },
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
    // let two_char_ops = HashMap::from([("==", TokenType::IsEqual), ("!=", TokenType::NotEqual)]);
    // let one_char_ops = HashMap::from([('=', TokenType::Assign)]);
    // Match the next char in src
    match look_ahead(src, to_match) {
        // lookup the two char op table to get TokenType
        true => match &src[..2] {
            "==" | "!=" => (
                consume_token(src, 2, Type::None, line_number, line_index),
                2,
            ),
            // If none, add invalid (for now)
            _ => panic!("Unrecognised 2 char op at {}, {}", line_number, line_index),
        },

        false => match &src[..1] {
            "=" => (
                consume_token(src, 1, Type::None, line_number, line_index),
                1,
            ),
            // If none, add invalid (for now)
            _ => panic!("Invalid char at {} {}", line_number, line_index),
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

fn parse_tokens(tokens: &mut VecDeque<Token>) -> VecDeque<Statement> {
    let mut statement_list = VecDeque::<Statement>::new();

    // while tokens.len() != 0 {
    //     statement_list.push_back(parse_statement(tokens))
    // }

    ast_pretty_printer(&parse_expression(tokens));

    statement_list
}

// fn parse_statement(tokens: &mut VecDeque<Token>) -> Statement {
//     if let Some(token) = tokens.pop_front() {
//         match token.lexeme.as_str() {
//             "let" => {
//                 match tokens.pop_front().expect("Should always be another token here") {
//                     TokenType::KeyWord => {

//                     }
//                     _ =>
//                 }
//             }
//             _ => panic!("This would be an error because we only have LET now"),
//         }
//     } else {
//         panic!("How is there a statement with no token??")
//     }
// }

fn parse_expression(tokens: &mut VecDeque<Token>) -> Expression {
    parse_equality(tokens)
}

fn parse_equality(tokens: &mut VecDeque<Token>) -> Expression {
    let mut expr = parse_comparision(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme.as_str() {
            "==" | "!=" => {
                expr =
                    Expression::Binary(Box::new(expr), token, Box::new(parse_comparision(tokens)));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    expr
}

fn parse_comparision(tokens: &mut VecDeque<Token>) -> Expression {
    let mut expr = parse_term(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme.as_str() {
            "<" | ">" => {
                expr = Expression::Binary(Box::new(expr), token, Box::new(parse_term(tokens)));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    expr
}

fn parse_term(tokens: &mut VecDeque<Token>) -> Expression {
    let mut expr = parse_factor(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme.as_str() {
            "+" | "-" => {
                expr = Expression::Binary(Box::new(expr), token, Box::new(parse_factor(tokens)));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    expr
}

fn parse_factor(tokens: &mut VecDeque<Token>) -> Expression {
    let mut expr = parse_unary(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme.as_str() {
            "/" | "*" => {
                expr = Expression::Binary(Box::new(expr), token, Box::new(parse_unary(tokens)));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    expr
}

fn parse_unary(tokens: &mut VecDeque<Token>) -> Expression {
    let first_token = &tokens[0];
    match first_token.lexeme.as_str() {
        "-" => Expression::Unary(
            tokens
                .pop_front()
                .expect("Should always be at least 1 element in tokens"),
            Box::new(parse_expression(tokens)),
        ),
        _ => parse_primary(tokens),
    }
}

fn parse_primary(tokens: &mut VecDeque<Token>) -> Expression {
    if let Some(token) = tokens.pop_front() {
        match token._type {
            Type::Bool | Type::Int => Expression::Literal(token),
            _ => match token.lexeme.as_str() {
                "(" => {
                    let left = token;
                    let expr = parse_expression(tokens);
                    if let Some(next_token) = tokens.pop_front() {
                        match next_token.lexeme.as_str() {
                            ")" => Expression::Group(left, Box::new(expr), next_token),
                            _ => panic!("Didn't match group"),
                        }
                    } else {
                        panic!("ran out of tokens before group finished")
                    }
                }
                _ => panic!("unrecognised primary token!"),
            },
        }
    } else {
        panic!("No primary tokens!")
    }
}

fn ast_pretty_printer(expr: &Expression) {
    match expr {
        Expression::Binary(left, op, right) => {
            print!("( ");

            ast_pretty_printer(left);
            print!(" {} ", op.lexeme);
            ast_pretty_printer(right);
            print!(" )");
        }
        Expression::Unary(op, right) => {
            print!("{}", op.lexeme);
            ast_pretty_printer(right);
        }
        Expression::Literal(token) => {
            print!("{}", &token.lexeme);
        }
        Expression::Group(_, inner_expr, _) => {
            print!("[ group ");
            ast_pretty_printer(inner_expr);
            print!(" ]");
        }
    }
}

#[derive(Debug, Clone)]
enum Statement {
    Assignment(Type, Token, Expression),
}

#[derive(Debug, Clone)]
enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Token),
    Group(Token, Box<Expression>, Token),
}

// // This is temporary as in future will build from parse tree
// fn build_asm(mut token_list: Vec<Token>) -> String {
//     let mut asm_code = String::new();
//     while token_list.len() != 0 {
//         match token_list[0].token {
//             // make this whole hierarchy better
//             TokenType::Exit => {
//                 if token_list.len() < 3 {
//                     asm_code.push_str("not enough tokens idiot!");
//                     break;
//                 }
//                 token_list.remove(0); // move removes to end
//                 let should_be_semi = token_list.remove(1);
//                 match should_be_semi.token {
//                     TokenType::Semi => asm_code.push_str(
//                         format_return(&token_list.remove(0).value.clone().expect("value")).as_str(),
//                     ),
//                     _ => asm_code.push_str("Should be a semicolon!"),
//                 }
//             }

//             _ => asm_code.push_str("Unrecognised Syntax!"),
//         }
//     }
//     asm_code
// }

// fn format_return(return_value: &str) -> String {
//     format!(
//         r#"global _start

// section .text

// _start:
//     mov rax, 60
//     mov rdi, {return_value}
//     syscall"#
//     )
// }
