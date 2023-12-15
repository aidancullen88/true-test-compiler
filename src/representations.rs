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

    pub fn line_info(&self) -> (&usize, &usize) {
        (&self.line_number, &self.line_index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier,
    Keyword,
    Literal,
    Operator,
    Assignment,
    Symbol,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(Type, String, Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Token),
    Group(Token, Box<Expression>, Token),
}

#[derive(Debug)]
pub struct Symbol {
    pub stack_offset: Option<u32>,
    pub _type: Type,
}

#[derive(Debug)]
pub enum InnerAddrType {
    Reg(String),
    Stack,
    StackOffset(u32),
}
