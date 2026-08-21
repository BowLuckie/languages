use crate::{
    ast::{Expr, Stmt, TypedExpr},
    types::Type::{self},
};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./grammer.pest"]
pub struct RuleParser;

pub fn parse(source: &str) -> Result<Vec<Stmt>, String> {
    let pairs =
        RuleParser::parse(Rule::Program, source).map_err(|err| format!("parser err {}", err))?;

    let mut program = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::Stmt {
            program.push(parse_stmt(pair)?);
        }
    }
    Ok(program)
}

fn parse_stmt(pair: Pair<Rule>) -> Result<Stmt, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::Function => parse_function(inner),
        Rule::Return => parse_return(inner),
        Rule::Assignment => parse_assignment(inner),
        Rule::Expr => Ok(Stmt::Expr(parse_expr(inner)?)),
        Rule::Conditional | Rule::WhileLoop | Rule::Comparison => {
            Ok(Stmt::Expr(parse_expr(inner)?))
        }
        r => Err(format!("Unexpected statement rule: {:?}", r)),
    }
}

fn parse_assignment(pair: Pair<Rule>) -> Result<Stmt, String> {
    // Assignment = { Identifier ~ (":" ~ Type)? ~ "=" ~ Expr }
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut type_ann = None;
    let mut value = None;

    for pair in inner {
        match pair.as_rule() {
            Rule::Type => type_ann = Some(parse_type(pair)?),
            Rule::Expr => value = Some(parse_expr(pair)?),
            _ => unreachable!(),
        }
    }

    let value = value.unwrap();

    Ok(Stmt::Assignment {
        name,
        type_ann,
        value,
    })
}

fn parse_type(pair: Pair<Rule>) -> Result<Type, String> {
    match pair.as_rule() {
        Rule::Type => parse_type(pair.into_inner().next().unwrap()),
        Rule::Int => Ok(Type::Int),
        Rule::Bool => Ok(Type::Bool),
        r => Err(format!("Unexpected type rule: {:?}", r)),
    }
}

fn parse_return(pair: Pair<Rule>) -> Result<Stmt, String> {
    // Return = { "return" ~ Expr }
    let expr = pair.into_inner().next().unwrap();
    Ok(Stmt::Return(parse_expr(expr)?))
}

fn parse_function(pair: Pair<Rule>) -> Result<Stmt, String> {
    // Function = {"fn" ~ Identifier ~ "(" ~ TypedParams? ~ ")" ~ ReturnType? ~ Block }
    // TypedParams = _{ TypedParam ~ ("," ~ TypedParam)* }
    // TypedParam  =  { Identifier ~ ":" ~ Type }
    // ReturnType  =  { "->" ~ Type }
    assert_eq!(pair.as_rule(), Rule::Function);

    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut params = Vec::new();
    let return_type = Type::Unknown;
    let body = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::TypedParam => {
                let mut param_inner = item.into_inner();
                let param_name = param_inner.next().unwrap().as_str().to_string();
                let param_type = parse_type(param_inner.next().unwrap())?;
                params.push((param_name, param_type));
            }
            Rule::ReturnType => {}
            Rule::Block => {}
            _ => unreachable!(),
        }
    }

    Ok(Stmt::Function {
        name,
        params,
        return_type,
        body,
    })
}

fn parse_expr(pair: Pair<Rule>) -> Result<TypedExpr, String> {
    let expr = match pair.as_rule() {
        Rule::Expr => {
            let inner = pair.into_inner().next().unwrap();
            return parse_expr(inner);
        }

        Rule::Conditional => parse_conditional(pair)?,
        Rule::WhileLoop => parse_while(pair)?,
        Rule::Comparison => parse_binary(pair)?,
        Rule::Additive => parse_binary(pair)?,
        Rule::Multiplicative => parse_binary(pair)?,
        Rule::Unary => return parse_unary(pair),
        Rule::Call => return parse_call(pair),
        Rule::Literal => parse_literal(pair)?,
        Rule::Int => Expr::Int(pair.as_str().parse().unwrap()),
        Rule::Bool => Expr::Bool(pair.as_str() == "true"),
        Rule::Identifier => Expr::Var(pair.as_str().to_string()),
        Rule::Block => {
            let stmts = parse_block(pair)?;
            Expr::Block(stmts)
        }
        r => return Err(format!("Unexpected expression rule: {:?}", r)),
    };

    Ok(TypedExpr::unknown(expr))
}

fn parse_literal(pair: Pair<Rule>) -> Result<Expr, String> {
    // Literal =  { Bool | Int }
    // Int     = @{ ASCII_DIGIT+ }
    // Bool    = @{ "true" | "false" }
    assert_eq!(pair.as_rule(), Rule::Literal);
    let mut inner = pair.into_inner();
    match inner.next().unwrap().as_rule() {
        Rule::Int => Ok(Expr::Int(inner.as_str().parse().unwrap())),
        Rule::Bool => Ok(Expr::Bool(inner.as_str() == "true")),
        _ => unreachable!(),
    }
}

fn parse_call(pair: Pair<Rule>) -> Result<TypedExpr, String> {
    // Call     =  {  Literal | Identifier | "(" ~ Expr ~ ")"  ~ CallArgs* }
    // CallArgs =  { "(" ~ ( Expr ~ ("," ~ Expr)* )? ~ ")" }
    // Args     = _{ Expr ~ ("," ~ Expr)* }
    // Primary = _{ Literal | Identifier | "(" ~ Expr ~ ")" }
    assert_eq!(pair.as_rule(), Rule::Call);
    let mut inner = pair.into_inner();
    let primary = inner.next().unwrap();
    let expr = parse_expr(inner.next().unwrap())?;
    Ok(expr)
}

fn parse_unary(pair: Pair<Rule>) -> Result<TypedExpr, String> {
    todo!()
}

fn parse_binary(pair: Pair<Rule>) -> Result<Expr, String> {
    todo!()
}

fn parse_while(pair: Pair<Rule>) -> Result<Expr, String> {
    todo!()
}

fn parse_conditional(pair: Pair<Rule>) -> Result<Expr, String> {
    todo!()
}

fn parse_block(pair: Pair<Rule>) -> Result<Vec<Stmt>, String> {
    todo!()
}
