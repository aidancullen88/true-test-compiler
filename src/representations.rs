#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Token {
    _type: Type,
    token_type: TokenType,
    lexeme: String,
    line_number: usize,
    line_index: usize,
}

impl Token {
    pub fn new(
        _type: Type,
        token_type: TokenType,
        lexeme: String,
        line_number: usize,
        line_index: usize,
    ) -> Self {
        Self {
            _type,
            token_type,
            lexeme,
            line_number,
            line_index,
        }
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn _type(&self) -> &Type {
        &self._type
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn line_number(&self) -> &usize {
        &self.line_number
    }

    pub fn line_index(&self) -> &usize {
        &self.line_index
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    None,
    Pointer(Box<Type>),
    Array(Box<Type>, u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier,
    Keyword,
    Literal,
    Operator,
    Assignment,
    Terminal,
}

#[derive(Debug, Clone)]
pub enum Block {
    Statement(Statement),
    Block(Statement, Box<Block>),
}

#[derive(PartialEq)]
pub enum Context {
    While,
    None,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(Assignment, Expression),
    If(Expression, Box<Statement>),
    IfElse(Expression, Box<Statement>, Box<Statement>),
    Block(Box<Block>),
    While(Expression, Box<Statement>),
    Break,
}

#[derive(Clone, Debug)]
pub enum Assignment {
    Value(Type, String),
    Pointer(Type, String),
    Mutation(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Literal),
    Group(Token, Box<Expression>, Token),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(Token),
    Bool(Token),
    Symbol(Token),
    List(Box<List>),
}

#[derive(Debug, Clone)]
pub enum List {
    Literal(Literal),
    List(Literal, Box<List>),
}

#[derive(Debug)]
pub struct Symbol {
    pub stack_offset: Option<u64>,
    pub _type: Type,
    pub mutable: bool,
    pub init_line: usize,
    pub last_ref: usize,
}

#[derive(Debug)]
pub enum InnerAddrType {
    Reg(String),
    Stack,
    StackOffset(u64),
}

impl InnerAddrType {
    pub fn is_memory(&self) -> bool {
        match self {
            InnerAddrType::StackOffset(_) => true,
            _ => false,
        }
    }
}
