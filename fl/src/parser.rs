use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};

type SResult<T> = Result<T, String>;
type ExprResult = SResult<Expr>;
type StmtResult = SResult<Stmt>;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RuleParser;

pub fn parse(source: &str) -> SResult<Program> {
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

fn parse_stmt(pair: Pair<Rule>) -> StmtResult {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::Function => parse_function(inner),
        Rule::Return => parse_return(inner),
        Rule::Assignment => parse_assignment(inner),
        Rule::Expr | Rule::Conditional | Rule::WhileLoop | Rule::Comparison => {
            Ok(Stmt::Expr(parse_expr(inner)?))
        }
        _ => unreachable!(),
    }
}

fn parse_assignment(pair: Pair<Rule>) -> StmtResult {
    assert_eq!(pair.as_rule(), Rule::Assignment);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expr(inner.next().unwrap())?;
    Ok(Stmt::Assignment { name, value })
}

fn parse_function(pair: Pair<Rule>) -> StmtResult {
    assert_eq!(pair.as_rule(), Rule::Function);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut params = Vec::new();
    let mut body = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::Identifier => {
                params.push(item.as_str().to_string());
            }
            Rule::Block => {
                body = parse_block(item)?;
            }
            _ => {}
        }
    }

    Ok(Stmt::Function { name, params, body })
}

fn parse_block(pair: Pair<Rule>) -> Result<Vec<Stmt>, String> {
    let mut stmts = Vec::new();
    for item in pair.into_inner() {
        if item.as_rule() == Rule::Stmt {
            stmts.push(parse_stmt(item)?);
        }
    }
    Ok(stmts)
}

fn parse_return(pair: Pair<Rule>) -> StmtResult {
    let expr = pair.into_inner().next().unwrap();
    Ok(Stmt::Return(parse_expr(expr)?))
}

fn parse_expr(pair: Pair<Rule>) -> ExprResult {
    match pair.as_rule() {
        Rule::Expr => {
            let inner = pair.into_inner().next().unwrap();
            parse_expr(inner)
        }
        Rule::Conditional => parse_conditional(pair),
        Rule::WhileLoop => parse_while(pair),
        Rule::Comparison => parse_binary(pair),
        Rule::Additive => parse_binary(pair),
        Rule::Multiplicative => parse_binary(pair),
        Rule::Unary => parse_unary(pair),
        Rule::Call => parse_call(pair),
        Rule::Literal => parse_literal(pair),
        Rule::Int => Ok(Expr::Int(pair.as_str().parse().unwrap())),
        Rule::Bool => Ok(Expr::Bool(pair.as_str() == "true")),
        Rule::Identifier => Ok(Expr::Var(pair.as_str().to_string())),
        Rule::Block => {
            let stmts = parse_block(pair)?;
            Ok(Expr::Block(stmts))
        }
        r => Err(format!("Unexpected expression rule: {:?}", r)),
    }
}

fn parse_literal(pair: Pair<Rule>) -> Result<Expr, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::Int => Ok(Expr::Int(inner.as_str().parse::<i64>().unwrap())),
        Rule::Bool => Ok(Expr::Bool(inner.as_str() == "true")),
        l => panic!("Unexpected Literal {:?}", l),
    }
}

fn parse_call(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut inner = pair.into_inner();
    let primrary = inner.next().unwrap();
    let mut expr = parse_expr(primrary)?;

    for call_arg in inner {
        if call_arg.as_rule() == Rule::CallArgs {
            let args: Vec<Expr> = call_arg
                .into_inner()
                .map(|e| parse_expr(e))
                .collect::<Result<_, _>>()?;
            if let Expr::Var(name) = expr {
                if name == "print" {
                    expr = Expr::Print(args)
                } else {
                    expr = Expr::Call { name, args };
                }
            } else {
                return Err("Can only call named functions".to_string());
            }
        }
    }

    Ok(expr)
}

fn parse_unary(pair: Pair<Rule>) -> Result<Expr, String> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::UnaryOp => {
            let op = match first.as_str() {
                "!" => UnaryOp::Not,
                "-" => UnaryOp::Neg,
                s => return Err(format!("Unknown unary operator: {}", s)),
            };
            let expr = parse_expr(inner.next().unwrap())?;
            Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
            })
        }
        _ => parse_expr(first),
    }
}

fn parse_binary(pair: Pair<Rule>) -> ExprResult {
    let mut inner = pair.into_inner();
    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "%" => BinaryOp::Mod,
            "<" => BinaryOp::Lt,
            ">" => BinaryOp::Gt,
            "<=" => BinaryOp::Le,
            ">=" => BinaryOp::Ge,
            "==" => BinaryOp::Eq,
            "!=" => BinaryOp::Ne,
            s => return Err(format!("Unknown operator: {}", s)),
        };

        let right = parse_expr(inner.next().unwrap())?;
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_conditional(pair: Pair<Rule>) -> ExprResult {
    assert_eq!(pair.as_rule(), Rule::Conditional);
    let mut inner = pair.into_inner();
    let cond = Box::new(parse_expr(inner.next().unwrap())?);
    let then_branch = parse_block(inner.next().unwrap())?;
    let else_branch = parse_block(inner.next().unwrap())?;
    Ok(Expr::If {
        cond,
        then_branch,
        else_branch,
    })
}

fn parse_while(pair: Pair<Rule>) -> ExprResult {
    let mut inner = pair.into_inner();
    let cond = Box::new(parse_expr(inner.next().unwrap())?);
    let body = parse_block(inner.next().unwrap())?;
    Ok(Expr::While { cond, body })
}
