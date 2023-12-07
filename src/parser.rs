use crate::representations::{Expression, Statement, Token, Type};

use std::collections::VecDeque;

pub fn parse_tokens(tokens: &mut VecDeque<Token>) -> VecDeque<Statement> {
    let mut statement_list = VecDeque::<Statement>::new();

    while tokens.len() != 0 {
        statement_list.push_back(parse_statement(tokens))
    }

    statement_list
}

fn parse_statement(tokens: &mut VecDeque<Token>) -> Statement {
    if let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "let" => {
                let (statement_type, identifier) = parse_identifier(tokens);
                match tokens.pop_front().expect("should be a token here").lexeme() {
                    "=" => {
                        let (expr, expr_type) = parse_expression(tokens);
                        if statement_type == expr_type {
                            match tokens.pop_front().expect("Should be a ; here").lexeme() {
                                ";" => Statement::Assignment(statement_type, identifier, expr),
                                _ => panic!("Should end with a ;"),
                            }
                        } else {
                            panic!("Expression and statement have different types!")
                        }
                    }
                    _ => panic!("Should only be an = here"),
                }
            }
            _ => panic!("This would be an error because we only have LET now"),
        }
    } else {
        panic!("How is there a statement with no token??")
    }
}

fn parse_identifier(tokens: &mut VecDeque<Token>) -> (Type, String) {
    if let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "int" => (
                Type::Int,
                tokens
                    .pop_front()
                    .expect("Should always be an id")
                    .lexeme()
                    .to_string(),
            ),
            "bool" => (
                Type::Bool,
                tokens
                    .pop_front()
                    .expect("Should always be an id")
                    .lexeme()
                    .to_string(),
            ),
            _ => panic!("there's no other types"),
        }
    } else {
        panic!("Whoops, no id to parse")
    }
}

fn parse_expression(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    parse_equality(tokens)
}

fn parse_equality(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_comparision(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "==" | "!=" => {
                let (right_expr, right_type) = parse_comparision(tokens);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else if _type == Type::Bool && right_type == Type::Bool {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    panic!("unsupported types for equality; expression")
                }
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    (expr, _type)
}

fn parse_comparision(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_term(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "<" | ">" => {
                let (right_expr, right_type) = parse_term(tokens);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    panic!("unsupported types for comparision expression")
                }
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    (expr, _type)
}

fn parse_term(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_factor(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "+" | "-" => {
                let (right_expr, right_type) = parse_factor(tokens);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Int;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    panic!("unsupported types for term expression")
                }
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    (expr, _type)
}

fn parse_factor(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_unary(tokens);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "/" | "*" => {
                let (right_expr, right_type) = parse_unary(tokens);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Int;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    panic!("unsupported types for factor expression")
                }
            }
            _ => {
                tokens.push_front(token);
                break;
            }
        }
    }

    (expr, _type)
}

fn parse_unary(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    let first_token = &tokens[0];
    match first_token.lexeme() {
        "-" => {
            let op = tokens.pop_front().expect("Should be at least 1 element");
            let (expr, _type) = parse_expression(tokens);
            (Expression::Unary(op, Box::new(expr)), _type)
        }
        _ => parse_primary(tokens),
    }
}

fn parse_primary(tokens: &mut VecDeque<Token>) -> (Expression, Type) {
    if let Some(token) = tokens.pop_front() {
        match token._type() {
            Type::Bool | Type::Int => {
                let _type = token._type().clone();
                (Expression::Literal(token), _type)
            }
            _ => match token.lexeme() {
                "(" => {
                    let left = token;
                    let (expr, _type) = parse_expression(tokens);
                    if let Some(next_token) = tokens.pop_front() {
                        match next_token.lexeme() {
                            ")" => (Expression::Group(left, Box::new(expr), next_token), _type),
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
