use crate::representations::{Expression, Statement, Symbol, Token, TokenType, Type};

use std::collections::{HashMap, VecDeque};

pub fn parse_tokens(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> VecDeque<Statement> {
    let mut statement_list = VecDeque::<Statement>::new();

    while tokens.len() != 0 {
        statement_list.push_back(parse_statement(tokens, symbol_table))
    }

    statement_list
}

fn parse_statement(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> Statement {
    if let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "let" => {
                let (statement_type, identifier) = parse_identifier(tokens);
                match tokens.pop_front().expect("should be a token here").lexeme() {
                    "=" => {
                        let (expr, expr_type) = parse_expression(tokens, symbol_table);
                        // Type check the var declaration with the expression
                        if statement_type == expr_type {
                            match tokens.pop_front().expect("Should be a ; here").lexeme() {
                                ";" => {
                                    // TODO: Should check if id already exists as this would be an
                                    // error
                                    symbol_table.insert(
                                        identifier.clone(),
                                        Symbol {
                                            stack_offset: None,
                                            _type: statement_type.clone(),
                                        },
                                    );
                                    Statement::Assignment(statement_type, identifier, expr)
                                }
                                _ => panic!("Should end with a ;"),
                            }
                        } else {
                            // Type error: needs handling
                            panic!("Expression and statement have different types!")
                        }
                    }
                    // parse error: needs handling
                    _ => panic!("Should only be an = here"),
                }
            }
            // Parse error: needs handling
            _ => panic!("This would be an error because we only have LET now"),
        }
    } else {
        // Actual error as statement should never be called without at least 1 token!
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
                    // Parse error: statment needs identifier
                    .expect("Should always be an id")
                    .lexeme()
                    .to_string(),
            ),
            "bool" => (
                Type::Bool,
                tokens
                    .pop_front()
                    // Parse error: statement needs identifier
                    .expect("Should always be an id")
                    .lexeme()
                    .to_string(),
            ),
            // Type error: unrecgonised type
            _ => panic!("Unrecognised type \"{}\" at {}:{}", token.lexeme(), token.line_info().0, token.line_info().1),
        }
    } else {
        // Parse error: can't have let without a token
        panic!("Whoops, no id to parse")
    }
}

fn parse_expression(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    parse_equality(tokens, symbol_table)
}

fn parse_equality(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_comparision(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "==" | "!=" => {
                let (right_expr, right_type) = parse_comparision(tokens, symbol_table);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else if _type == Type::Bool && right_type == Type::Bool {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    // Type error: mismatched types in expression
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

fn parse_comparision(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_term(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "<" | ">" => {
                let (right_expr, right_type) = parse_term(tokens, symbol_table);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    // Type error: mismatched types in expr
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

fn parse_term(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_factor(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "+" | "-" => {
                let (right_expr, right_type) = parse_factor(tokens, symbol_table);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Int;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    // Type error: mismatched types in expr
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

fn parse_factor(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_unary(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "/" | "*" => {
                let (right_expr, right_type) = parse_unary(tokens, symbol_table);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Int;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    // Type error: mismatched types in expr
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

fn parse_unary(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    let first_token = &tokens[0];
    match first_token.lexeme() {
        "-" => {
            // Parse error: can't have a unary op without a literal/id
            let op = tokens.pop_front().expect("Should be at least 1 element");
            let (expr, _type) = parse_expression(tokens, symbol_table);
            (Expression::Unary(op, Box::new(expr)), _type)
        }
        _ => parse_primary(tokens, symbol_table),
    }
}

fn parse_primary(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> (Expression, Type) {
    if let Some(token) = tokens.pop_front() {
        match token.token_type() {
            TokenType::Identifier => match symbol_table.get(token.lexeme()) {
                Some(symbol_info) => {
                    return (Expression::Literal(token), symbol_info._type.clone())
                }
                None => panic!("Unrecognised identifier {} in expr", token.lexeme()),
            },
            TokenType::Literal | TokenType::Symbol => {
                match token._type() {
                    Type::Bool | Type::Int => {
                        let _type = token._type().clone();
                        return (Expression::Literal(token), _type);
                    }
                    _ => match token.lexeme() {
                        "(" => {
                            let left = token;
                            let (expr, _type) = parse_expression(tokens, symbol_table);
                            if let Some(next_token) = tokens.pop_front() {
                                match next_token.lexeme() {
                                    ")" => {
                                        return (
                                            Expression::Group(left, Box::new(expr), next_token),
                                            _type,
                                        )
                                    }
                                    // Parse error: PAREN not closed
                                    _ => panic!("Didn't match group"),
                                }
                            } else {
                                // Parse error: PAREN not closed before EOF
                                panic!("ran out of tokens before group finished")
                            }
                        }
                        // Actual error: how is there another token type here?
                        _ => panic!("unrecognised primary token!"),
                    },
                }
            }
            _ => panic!("{} is not a valid primary token", token.lexeme()),
        }
    } else {
        // Parse error: expected a literal or id
        panic!("No primary tokens!")
    }
}
