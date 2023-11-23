//use clap::Parser;
use std::collections::HashMap;
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
    println!("{:#?}", lexed_line);

    parse_tokens(&mut lexed_line);

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
    NotEqual,
    True,
    False,
    Subtract,
    Add,
    Divide,
    Multiply,
    Greater,
    Less,
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    value: Option<Value>,
    lexeme: String,
    line_number: usize,
    line_index: usize,
}

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Int(i32),
}

fn lexer(mut src: String) -> VecDeque<Token> {
    let single_char_keys = HashMap::from([
        (';', TokenType::Semi),
        ('(', TokenType::LeftParen),
        (')', TokenType::RightParen),
        ('-', TokenType::Subtract),
        ('/', TokenType::Divide),
        ('*', TokenType::Multiply),
        ('+', TokenType::Add),
        ('>', TokenType::Greater),
        ('<', TokenType::Less),
    ]);
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
        match single_char_keys.get(&first_char) {
            Some(token_type) => {
                tokens.push_back(consume_token(
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
            },
        }
    }
    tokens
}

fn consume_token(
    src: &mut String,
    chars_to_consume: usize,
    token_type: TokenType,
    value: Option<Value>,
    line_number: usize,
    line_index: usize,
) -> Token {
    // remove the amount of characters specified from the string
    let lexeme = src.drain(..chars_to_consume);
    println!("lexeme is {}", &lexeme.as_str());
    // Return the token created
    Token {
        token_type: token_type.clone(),
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
                consume_token(
                    src,
                    lexume_index,
                    TokenType::IntLit,
                    Some(Value::Int(lexeme.to_string().parse::<i32>().expect(
                        "Should always be able to parse a number literal to a number",
                    ))),
                    line_number,
                    line_index,
                ),
                lexume_index,
            );
        }
        _ => match lexeme {
            "true" => (
                consume_token(
                    src,
                    lexume_index,
                    TokenType::True,
                    Some(Value::Bool(true)),
                    line_number,
                    line_index,
                ),
                lexume_index,
            ),
            "false" => (
                consume_token(
                    src,
                    lexume_index,
                    TokenType::False,
                    Some(Value::Bool(false)),
                    line_number,
                    line_index,
                ),
                lexume_index,
            ),
            _ => match lexume_keys.get(lexeme) {
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
            },
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
    let two_char_ops = HashMap::from([("==", TokenType::IsEqual), ("!=", TokenType::NotEqual)]);
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

fn parse_tokens(tokens: &mut VecDeque<Token>) {
    let mut expression_list = VecDeque::<Expression>::new();

    while tokens.len() != 0 {
        expression_list.push_back(parse_expression(tokens))
    }

    println!("{:#?}", expression_list);
}

fn parse_expression(tokens: &mut VecDeque<Token>) -> Expression {
    Expression::Equality(parse_equality(tokens))
}

fn parse_equality(tokens: &mut VecDeque<Token>) -> Equality {
    let mut left = EqualityChoice::Comparison(parse_comparision(tokens));
    let mut operator: Option<Token> = None;
    let mut right: Option<Comparison> = None;
    while let Some(token) = tokens.pop_front() {
        match token.token_type {
            TokenType::IsEqual | TokenType::NotEqual => {
                left = EqualityChoice::Equality(Equality {
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: right.clone(),
                });
                operator = Some(token);
                right = Some(parse_comparision(tokens));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }
    Equality {
        left: Box::new(left),
        operator,
        right,
    }
}

fn parse_comparision(tokens: &mut VecDeque<Token>) -> Comparison {
    let mut left = ComparisonChoice::Term(parse_term(tokens));
    let mut operator: Option<Token> = None;
    let mut right: Option<Term> = None;
    while let Some(token) = tokens.pop_front() {
        match token.token_type {
            TokenType::Greater | TokenType::Less => {
                left = ComparisonChoice::Comparison(Comparison {
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: right.clone(),
                });
                operator = Some(token);
                right = Some(parse_term(tokens));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }
    Comparison {
        left: Box::new(left),
        operator,
        right,
    }
}

fn parse_term(tokens: &mut VecDeque<Token>) -> Term {
    let mut left = TermChoice::Factor(parse_factor(tokens));
    let mut operator: Option<Token> = None;
    let mut right: Option<Factor> = None;
    while let Some(token) = tokens.pop_front() {
        match token.token_type {
            TokenType::Add | TokenType::Subtract => {
                left = TermChoice::Term(Term {
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: right.clone(),
                });
                operator = Some(token);
                right = Some(parse_factor(tokens));
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }
    Term {
        left: Box::new(left),
        operator,
        right,
    }
}

fn parse_factor(tokens: &mut VecDeque<Token>) -> Expression {
    let mut expr = parse_unary(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.token_type {
            TokenType::Divide | TokenType::Multiply => {
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
    match first_token.token_type {
        TokenType::Subtract => Expression::Unary(
            tokens
                .pop_front()
                .expect("Should always be at least 1 element in tokens"),
            Box::new(parse_expression(tokens)),
        ),

        _ => parse_literal(tokens),
    }
}

fn parse_literal(tokens: &mut VecDeque<Token>) -> Expression {
    let first_token = &tokens[0];
    match first_token.token_type {
        TokenType::True | TokenType::False => {
            let literal_value: bool;
            if let Value::Bool(lv) = tokens
                .pop_front()
                .expect("Should always be at least one element in the queue")
                .value
                .expect("Bool tokentype should always have a value")
            {
                literal_value = lv;
            } else {
                panic!("Found int value in bool token")
            }
            Expression::Literal(LiteralType::Bool(literal_value))
        }
        TokenType::IntLit => {
            let literal_value: i32;
            if let Value::Int(lv) = tokens
                .pop_front()
                .expect("Should always be at least 1 element in the queue")
                .value
                .expect("IntLit tokentype should always have a value")
            {
                literal_value = lv;
            } else {
                panic!("Found bool in int token")
            }
            Expression::Literal(LiteralType::Int(literal_value))
        }
        _ => panic!("Whoops! can't deal with this yet"),
    }
}

#[derive(Debug, Clone)]
enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(LiteralType),
}

#[derive(Debug, Clone)]
enum LiteralType {
    Int(i32),
    Bool(bool),
}

// #[derive(Debug, Clone)]
// struct Equality {
//     left: Box<EqualityChoice>,
//     operator: Option<Token>,
//     right: Option<Comparison>,
// }

// #[derive(Debug, Clone)]
// enum EqualityChoice {
//     Equality(Equality),
//     Comparison(Comparison),
// }

// #[derive(Debug, Clone)]
// struct Comparison {
//     left: Box<ComparisonChoice>,
//     operator: Option<Token>,
//     right: Option<Term>,
// }

// #[derive(Debug, Clone)]
// enum ComparisonChoice {
//     Comparison(Comparison),
//     Term(Term),
// }

// #[derive(Debug, Clone)]
// struct Term {
//     left: Box<TermChoice>,
//     operator: Option<Token>,
//     right: Option<Factor>,
// }

// #[derive(Debug, Clone)]
// enum TermChoice {
//     Term(Term),
//     Factor(Factor),
// }

// #[derive(Debug, Clone)]
// struct Factor {
//     left: Box<FactorChoice>,
//     operator: Option<Token>,
//     right: Option<Unary>,
// }

// #[derive(Debug, Clone)]
// enum FactorChoice {
//     Factor(Factor),
//     Unary(Unary),
// }

// #[derive(Debug, Clone)]
// struct Unary {
//     operator: Option<Token>,
//     right: Box<UnaryChoice>,
// }

// #[derive(Debug, Clone)]
// enum UnaryChoice {
//     Unary(Unary),
//     Primary(Primary),
// }

// #[derive(Debug, Clone)]
// enum Primary {
//     Literal(Literal),
//     // Grouping(Grouping),
// }

// // struct Grouping {
// //     left: Token,
// //     expression: Expression,
// //     right: Token,
// // }

// #[derive(Debug, Clone)]
// enum Literal {
//     Bool(bool),
//     Int(isize),
// }

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
