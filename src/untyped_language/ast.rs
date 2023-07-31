use crate::parser_combinator::Token;

#[derive(Debug)]
pub enum Value {
    Number(i32),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Parameter(pub String, pub String); //name, type

#[derive(Debug)]
pub struct Fun {
    pub name: Token<String>,
    pub params: Vec<Token<Parameter>>,
}
