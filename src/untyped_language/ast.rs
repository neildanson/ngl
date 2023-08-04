use crate::parser_combinator::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    Ident(String), //should this be token<string>?
    Call(Token<String>, Vec<Token<Expr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(Token<String>, Token<Expr>),
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
    pub return_type: Token<String>,
}
