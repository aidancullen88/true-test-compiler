use crate::representations::{
    Assignment, Block, Context, Expression, List, Literal, Statement, Symbol, Token, TokenType,
    Type,
};

use core::panic;
use std::collections::{HashMap, VecDeque};

pub fn parse_tokens(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
) -> VecDeque<Statement> {
    let mut statement_list = VecDeque::<Statement>::new();

    while tokens.len() != 0 {
        statement_list.push_back(parse_statement(tokens, symbol_table, &Context::None))
    }

    statement_list
}

fn parse_block(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
    context: &Context,
) -> Block {
    match tokens.pop_front() {
        Some(token) => {
            tokens.push_front(token);
            let stmt = parse_statement(tokens, symbol_table, context);
            match lookahead(tokens, "}") {
                true => return Block::Statement(stmt),
                false => {
                    return Block::Block(
                        stmt,
                        Box::new(parse_block(tokens, symbol_table, context)),
                    );
                }
            }
        }
        None => panic!("block with no tokens!"),
    }
}

fn parse_statement(
    tokens: &mut VecDeque<Token>,
    symbol_table: &mut HashMap<String, Symbol>,
    context: &Context,
) -> Statement {
    if let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "const" => {
                let (statement_type, identifier) = parse_identifier(tokens);
                match tokens.pop_front().expect("should be a token here").lexeme() {
                    "=" => {
                        let (expr, expr_type) = parse_expression(tokens, symbol_table);
                        // Type check the var declaration with the expression
                        if statement_type == expr_type {
                            match tokens.pop_front().expect("Should be a ; here").lexeme() {
                                ";" => {
                                    if let Some(_) = symbol_table.get(&identifier) {
                                        panic!(
                                            "{} already exists and cannot be assigned again!",
                                            &identifier
                                        )
                                    } else {
                                        symbol_table.insert(
                                            identifier.clone(),
                                            Symbol {
                                                stack_offset: None,
                                                _type: statement_type.clone(),
                                                mutable: false,
                                                init_line: *token.line_number(),
                                                last_ref: *token.line_number(),
                                            },
                                        );
                                        let assign_type = match statement_type {
                                            Type::Pointer(_) => Assignment::Pointer(statement_type, identifier),
                                            _ => Assignment::Value(statement_type, identifier)
                                        };
                                        Statement::Assignment(assign_type, expr)
                                    }
                                }
                                _ => panic!("Should end with a ;"),
                            }
                        } else {
                            // Type error: needs handling
                            panic!(
                                "Expression and statement have different types! at line {}",
                                token.line_number()
                            )
                        }
                    }
                    // parse error: needs handling
                    _ => panic!("Should only be an = here"),
                }
            }
            "mut" => {
                let (stmt_type, id) = parse_identifier(tokens);
                match tokens
                    .pop_front()
                    .expect("Should be an equals here")
                    .lexeme()
                {
                    "=" => {
                        let (expr, expr_type) = parse_expression(tokens, symbol_table);
                        if stmt_type == expr_type {
                            match tokens
                                .pop_front()
                                .expect("missing semicolon unexpected EOF")
                                .lexeme()
                            {
                                ";" => {
                                    if let Some(_) = symbol_table.get(&id) {
                                        panic!("Cannot re-assign var {}", &id)
                                    } else {
                                        symbol_table.insert(
                                            id.clone(),
                                            Symbol {
                                                stack_offset: None,
                                                _type: stmt_type.clone(),
                                                mutable: true,
                                                init_line: *token.line_number(),
                                                last_ref: *token.line_number(),
                                            },
                                        );
                                        let assign_type = match stmt_type {
                                            Type::Pointer(_) => Assignment::Pointer(stmt_type, id),
                                            _ => Assignment::Value(stmt_type, id)
                                        };
                                        Statement::Assignment(assign_type, expr)
                                    }
                                }
                                _ => panic!("Expected semicolon line {}", token.line_number()),
                            }
                        } else {
                            panic!(
                                "Statement type does not make expression type at {}",
                                token.line_number()
                            )
                        }
                    }
                    _ => panic!("Must be an equals for assignment"),
                }
            }
            "if" => {
                let (expr, expr_type) = parse_expression(tokens, symbol_table);
                if expr_type != Type::Bool {
                    panic!("Must be boolean expression!");
                }
                let if_block = parse_statement(tokens, symbol_table, context);
                match lookahead(tokens, "else") {
                    false => return Statement::If(expr, Box::new(if_block)),
                    true => {
                        tokens
                            .pop_front()
                            .expect("Already checked that a token exists");
                        let else_block = parse_statement(tokens, symbol_table, context);
                        return Statement::IfElse(expr, Box::new(if_block), Box::new(else_block));
                    }
                }
            }
            "while" => {
                let (expr, expr_type) = parse_expression(tokens, symbol_table);
                if expr_type != Type::Bool {
                    panic!(
                        "While requires boolean expression at line {}",
                        token.line_number()
                    )
                }
                let while_block = parse_statement(tokens, symbol_table, &Context::While);
                Statement::While(expr, Box::new(while_block))
            }
            "break" => {
                if context != &Context::While {
                    panic!(
                        "Break statements are only valid in while loops! at {}",
                        token.line_number()
                    )
                }
                match tokens
                    .pop_front()
                    .expect("Unexcepted EOF: expected ;")
                    .lexeme()
                {
                    ";" => return Statement::Break,
                    wrong => panic!(
                        "Expected ';', found {} at line {}",
                        wrong,
                        token.line_number()
                    ),
                }
            }
            "{" => {
                let block = parse_block(tokens, symbol_table, context);
                match tokens.pop_front() {
                    Some(token) => match token.lexeme() {
                        "}" => return Statement::Block(Box::new(block)),
                        _ => panic!(
                            "Should be a }} at {}:{}",
                            token.line_number(),
                            token.line_index()
                        ),
                    },
                    None => panic!("File ended without closing brace"),
                }
            }
            _ => match token.token_type() {
                TokenType::Identifier => match symbol_table.get(token.lexeme()) {
                    Some(symbol_info) => {
                        if !symbol_info.mutable {
                            panic!(
                                "Attempted to mutate const var {} at {}:{}",
                                token.lexeme(),
                                token.line_number(),
                                token.line_index()
                            );
                        }
                        match tokens
                            .pop_front()
                            .expect("Unexpected EOF: should be equals")
                            .lexeme()
                        {
                            "=" => {
                                let (expr, expr_type) = parse_expression(tokens, symbol_table);
                                if symbol_info._type == expr_type {
                                    match tokens
                                        .pop_front()
                                        .expect("Expected semicolon found EOF")
                                        .lexeme()
                                    {
                                        ";" => {
                                            symbol_table.insert(
                                                token.lexeme().to_string(),
                                                Symbol {
                                                    stack_offset: None,
                                                    _type: symbol_info._type.clone(),
                                                    mutable: true,
                                                    init_line: symbol_info.init_line,
                                                    last_ref: *token.line_number(),
                                                },
                                            );
                                            let assign_type =
                                                Assignment::Mutation(token.lexeme().to_string());
                                            Statement::Assignment(assign_type, expr)
                                        }
                                        wrong => panic!(
                                            "Expected ';', found {}, on line {}",
                                            wrong,
                                            token.line_number()
                                        ),
                                    }
                                } else {
                                    panic!(
                                        "var type does not match expr at {}",
                                        token.line_number()
                                    )
                                }
                            }
                            _ => panic!("Should only be equals on line {}", token.line_number()),
                        }
                    }
                    None => panic!(
                        "Attempted to mutate token {} before assignment {}:{}",
                        token.lexeme(),
                        token.line_number(),
                        token.line_index()
                    ),
                },
                _ => panic!(
                    "Statement cannot start with {:?} at {}:{}",
                    token.token_type(),
                    token.line_number(),
                    token.line_index()
                ),
            },
        }
    } else {
        // Actual error as statement should never be called without at least 1 token!
        panic!("How is there a statement with no token??")
    }
}

fn parse_identifier(tokens: &mut VecDeque<Token>) -> (Type, String) {
    if let Some(token) = tokens.pop_front() {
        let declared_type = parse_type(token.clone());
        if lookahead(tokens, "*") {
            tokens.pop_front();
            return (
                Type::Pointer(Box::new(declared_type)),
                tokens
                    .pop_front()
                    .expect("Unexpected EOF: expected symbol")
                    .lexeme()
                    .to_string(),
            );
        } else if lookahead(tokens, "[") {
            tokens.pop_front();
            let array_length_token = tokens
                .pop_front()
                .expect("Unexpected EOF: expected list length");
            let array_length: u64 = if array_length_token.token_type() == &TokenType::Literal {
                array_length_token.lexeme().parse().unwrap()
            } else {
                panic!("Array length specifier must be a literal!");
            };
            if tokens
                .pop_front()
                .expect("Unexpected EOF: expected ]")
                .lexeme()
                != "]"
            {
                panic!(
                    "Unexpected character in array type at line: {}",
                    token.line_number()
                )
            };
            return (
                Type::Pointer(Box::new(Type::Array(Box::new(declared_type), array_length))),
                tokens
                    .pop_front()
                    .expect("Unexpected EOF: expected symbol")
                    .lexeme()
                    .to_string(),
            );
        } else {
            return (
                declared_type,
                tokens
                    .pop_front()
                    .expect("Unexpected EOF: expected symbol")
                    .lexeme()
                    .to_string(),
            );
        }
    } else {
        // Parse error: can't have let without a token
        panic!("Whoops, no id to parse")
    }
}

fn parse_type(token: Token) -> Type {
    match token.lexeme() {
        "int" => Type::Int,
        "bool" => Type::Bool,
        // Type error: unrecgonised type
        _ => panic!(
            "Unrecognised type \"{}\" at {}:{}",
            token.lexeme(),
            token.line_number(),
            token.line_index()
        ),
    }
}

fn parse_expression(
    tokens: &mut VecDeque<Token>,
    symbol_table: &HashMap<String, Symbol>,
) -> (Expression, Type) {
    parse_equality(tokens, symbol_table)
}

fn parse_equality(
    tokens: &mut VecDeque<Token>,
    symbol_table: &HashMap<String, Symbol>,
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
    symbol_table: &HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_term(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "<" | ">" | "<=" | ">=" => {
                let (right_expr, right_type) = parse_term(tokens, symbol_table);
                if _type == Type::Int && right_type == Type::Int {
                    _type = Type::Bool;
                    expr = Expression::Binary(Box::new(expr), token, Box::new(right_expr));
                } else {
                    // Type error: mismatched types in expr
                    panic!(
                        "unsupported types {:?}:{:?} for comparision expression",
                        _type, right_type
                    )
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
    symbol_table: &HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_factor(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "+" | "-" => {
                let (right_expr, right_type) = parse_factor(tokens, symbol_table);
                println!("{:#?} == {:#?}", _type, right_type);
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
    symbol_table: &HashMap<String, Symbol>,
) -> (Expression, Type) {
    let (mut expr, mut _type) = parse_unary(tokens, symbol_table);
    while let Some(token) = tokens.pop_front() {
        match token.lexeme() {
            "/" | "*" | "%" => {
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
    symbol_table: &HashMap<String, Symbol>,
) -> (Expression, Type) {
    let first_token = &tokens[0];

    match first_token.lexeme() {
        "-" => {
            let op = tokens.pop_front().expect("Should be op here");
            let (expr, _type) = parse_unary(tokens, symbol_table);
            (Expression::Unary(op, Box::new(expr)), _type)
        }
        "&" => {
            let op = tokens.pop_front().expect("Should be op here");
            let (expr, _type) = parse_unary(tokens, symbol_table);
            match &expr {
                Expression::Literal(lit) => match lit {
                    Literal::Symbol(_) => (),
                    _ => panic!("Must reference a variable in memory"),
                },
                _ => panic!("Cannot reference a temporary expression"),
            }
            let pointer_type = Type::Pointer(Box::new(_type));
            (Expression::Unary(op, Box::new(expr)), pointer_type)
        }
        "*" => {
            let op = tokens.pop_front().expect("Should be op here");
            let (expr, _type) = parse_unary(tokens, symbol_table);
            println!("inner unary type = {:#?}", _type);
            let inner_type = match _type {
                Type::Pointer(inner_type) => inner_type,
                wrong => panic!("{:?} cannot be dereferenced!", wrong),
            };
            (Expression::Unary(op, Box::new(expr)), *inner_type)
        }
        _ => parse_primary(tokens, symbol_table),
    }
}

fn parse_primary(
    tokens: &mut VecDeque<Token>,
    symbol_table: &HashMap<String, Symbol>,
) -> (Expression, Type) {
    if let Some(token) = tokens.pop_front() {
        match token.token_type() {
            TokenType::Identifier => match symbol_table.get(token.lexeme()) {
                Some(symbol_info) => {
                    return (
                        Expression::Literal(Literal::Symbol(token)),
                        symbol_info._type.clone(),
                    )
                }
                None => panic!("Unrecognised identifier {} in expr", token.lexeme()),
            },
            TokenType::Literal | TokenType::Terminal => {
                match token._type() {
                    Type::Bool => {
                        let _type = token._type().clone();
                        return (Expression::Literal(Literal::Bool(token)), _type);
                    }
                    Type::Int => {
                        let _type = token._type().clone();
                        return (Expression::Literal(Literal::Int(token)), _type);
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
                        "[" => {
                            println!("Parsing list!");
                            let (list, list_type, list_length) =
                                parse_list_literal(tokens, symbol_table);
                            println!("{:#?}", list);
                            match tokens
                                .pop_front()
                                .expect("Unexpected EOF: expected ']'")
                                .lexeme()
                            {
                                "]" => (
                                    Expression::Literal(Literal::List(Box::new(list))),
                                    Type::Pointer(Box::new(Type::Array(
                                        Box::new(list_type),
                                        list_length,
                                    ))),
                                ),
                                wrong => {
                                    panic!("Unexpected token {} at end of list literal", wrong)
                                }
                            }
                        }
                        wrong => panic!("unrecognised primary token '{}'", wrong),
                    },
                }
            }
            _ => panic!(
                "{} is not a valid primary token at {}:{}",
                token.lexeme(),
                token.line_number(),
                token.line_index()
            ),
        }
    } else {
        // Parse error: expected a literal or id
        panic!("No primary tokens!")
    }
}

fn parse_list_literal(
    tokens: &mut VecDeque<Token>,
    symbol_table: &HashMap<String, Symbol>,
) -> (List, Type, u64) {
    let (first_expr, lit_type) = parse_primary(tokens, symbol_table);
    let first_literal = match first_expr {
        Expression::Literal(first_literal) => first_literal,
        _ => panic!("List literal must only contain literals"),
    };
    let mut external_count = 1;
    if lookahead(tokens, "]") {
        return (List::Literal(first_literal), lit_type, external_count);
    }
    match tokens
        .pop_front()
        .expect("Unexpected EOF: expected ','")
        .lexeme()
    {
        "," => (),
        wrong => panic!("Unexpected token {} in list literal, expected ','", wrong),
    }
    let (next_list, next_list_type, internal_count) = parse_list_literal(tokens, symbol_table);
    external_count += internal_count;
    if lit_type != next_list_type {
        panic!("TypeError: List must be uniform types",)
    }
    return (
        List::List(first_literal, Box::new(next_list)),
        lit_type,
        external_count,
    );
}

fn lookahead(tokens: &VecDeque<Token>, match_lexeme: &str) -> bool {
    if let Some(token) = tokens.get(0) {
        println!("Token: {}, match: {}", token.lexeme(), match_lexeme);
        if token.lexeme() == match_lexeme {
            return true;
        } else {
            return false;
        }
    } else {
        false
    }
}
