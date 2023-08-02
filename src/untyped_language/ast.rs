use crate::parser_combinator::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(Token<String>, Token<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprOrStatement {
    Expr(Expr),
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter(pub Token<String>, pub Token<String>); //name, type

#[derive(Debug, Clone)]
pub struct Fun {
    pub name: Token<String>,
    pub params: Vec<Token<Parameter>>,
    pub body: Vec<Token<ExprOrStatement>>,
}
