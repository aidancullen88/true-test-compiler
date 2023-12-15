use std::collections::VecDeque;

use crate::representations::{Token, TokenType, Type};

pub fn lexer(mut src: String) -> VecDeque<Token> {
    let mut tokens = VecDeque::<Token>::new();
    // src_index keeps the total position in the src
    let mut src_index = 0;
    let src_length = src.len();
    let mut line_number = 1;
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
            ';' | '(' | ')' => {
                tokens.push_back(consume_token(
                    &mut src,
                    1,
                    Type::None,
                    TokenType::Symbol,
                    line_number,
                    line_index,
                ));
                src_index += 1;
                line_index += 1;
            }
            '+' | '-' | '/' | '*' | '<' | '>' => {
                tokens.push_back(consume_token(
                    &mut src,
                    1,
                    Type::None,
                    TokenType::Operator,
                    line_number,
                    line_index,
                ));
                src_index += 1;
                line_index += 1;
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
    token_type: TokenType,
    line_number: usize,
    line_index: usize,
) -> Token {
    // remove the amount of characters specified from the string
    let lexeme = src.drain(..chars_to_consume);
    /* println!("lexeme is {}", &lexeme.as_str()); */
    // Return the token created
    Token::new(
        _type,
        token_type,
        lexeme.as_str().to_string(),
        line_number,
        line_index,
    )
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
                consume_token(
                    src,
                    lexume_index,
                    Type::Int,
                    TokenType::Literal,
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
                    Type::Bool,
                    TokenType::Literal,
                    line_number,
                    line_index,
                ),
                lexume_index,
            ),
            "false" => (
                consume_token(
                    src,
                    lexume_index,
                    Type::Bool,
                    TokenType::Literal,
                    line_number,
                    line_index,
                ),
                lexume_index,
            ),
            "let" | "int" | "bool" => (
                consume_token(
                    src,
                    lexume_index,
                    Type::None,
                    TokenType::Keyword,
                    line_number,
                    line_index,
                ),
                lexume_index,
            ),
            _ => (
                consume_token(
                    src,
                    lexume_index,
                    Type::None,
                    TokenType::Identifier,
                    line_number,
                    line_index,
                ),
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
                consume_token(
                    src,
                    2,
                    Type::None,
                    TokenType::Operator,
                    line_number,
                    line_index,
                ),
                2,
            ),
            // If none, add invalid (for now)
            _ => panic!("Unrecognised 2 char op at {}, {}", line_number, line_index),
        },

        false => match &src[..1] {
            "=" => (
                consume_token(
                    src,
                    1,
                    Type::None,
                    TokenType::Assignment,
                    line_number,
                    line_index,
                ),
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
