#[derive(Debug, Clone)]
pub struct Token {
    _type: Type,
    lexeme: String,
    line_number: usize,
    line_index: usize,
}

impl Token {
    pub fn new(_type: Type, lexeme: String, line_number: usize, line_index: usize) -> Self {
        Self {
            _type,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    None,
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
