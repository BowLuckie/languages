//! Program = _{ SOI ~ Expr ~ EOF }
use pest::{Parser, iterators::Pair};

use crate::ast::{Node, Operator};

/// the CalcParser will call pest to lex and parse the input, then it will be turned into ast
/// Program = _{ SOI ~ Expr ~ EOF }
/// Expr = { BinaryExpr | UnaryExpr | Term }
/// Term = { Int | "(" ~ Expr ~ ")" }
/// UnaryExpr = { Operator ~ Term }
/// BinaryExpr = { (UnaryExpr | Term) ~ (Operator ~ Term)+ }
/// Operator = { "+" | "-" }
/// Int = @{ ASCII_DIGIT+ }
/// WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
/// EOF = _{ EOI | ";" }
#[derive(pest_derive::Parser)]
#[grammar = "./grammer.pest"]
struct CalcParser;

pub fn parse(source: &str) -> Result<Vec<Node>, pest::error::Error<Rule>> {
    let mut ast = vec![];

    // Program = _{ SOI ~ (Expr ~ EOF)* }
    let pairs = CalcParser::parse(Rule::Program, source)?;

    for pair in pairs {
        if let Rule::Expr = pair.as_rule() {
            ast.push(parse_expr(pair));
        }
    }

    Ok(ast)
}

fn parse_expr(pair: Pair<'_, Rule>) -> Node {
    match pair.as_rule() {
        Rule::Expr => parse_expr(pair.into_inner().next().unwrap()),
        Rule::BinaryExpr => {
            let mut children = pair.into_inner();

            let first = children.next().unwrap();

            let mut lhs = if first.as_rule() == Rule::UnaryExpr {
                let mut inner = first.into_inner();
                let op = parse_operator(inner.next().unwrap());
                let value = parse_term(inner.next().unwrap());
                fold_unary_expr(op, value)
            } else {
                parse_term(first)
            };

            while let Some(op) = children.next() {
                let op = parse_operator(op);
                let rhs = parse_term(children.next().unwrap());
                lhs = fold_binary_expr(op, lhs, rhs);
            }

            lhs
        }
        Rule::UnaryExpr => {
            let mut children = pair.into_inner();

            let op = parse_operator(children.next().unwrap());
            let value = parse_term(children.next().unwrap());

            fold_unary_expr(op, value)
        }
        Rule::Term => parse_term(pair),
        unknown => panic!("unknown term: {:?}", unknown),
    }
}

fn fold_binary_expr(op: Operator, lhs: Node, rhs: Node) -> Node {
    Node::BinaryExpr {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn parse_term(pair: Pair<'_, Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::Term);

    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::Int => {
            let int = child.as_str().parse::<i32>().unwrap();
            Node::Int(int)
        }
        Rule::Expr => parse_expr(child),
        _ => unreachable!(),
    }
}

fn fold_unary_expr(op: Operator, child: Node) -> Node {
    Node::UnaryExpr {
        op,
        child: Box::new(child),
    }
}

fn parse_operator(op: Pair<'_, Rule>) -> Operator {
    match op.as_str() {
        "+" => Operator::Plus,
        "-" => Operator::Min,
        "/" => Operator::Div,
        "*" => Operator::Mul,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        assert!(parse("b").is_err());

        let one = parse("1");
        assert!(one.is_ok());
        assert_eq!(one.unwrap()[0], Node::Int(1));
    }

    #[test]
    fn unary_expr() {
        let plus_one = parse("+1");
        assert!(plus_one.is_ok());
        assert_eq!(
            plus_one.clone().unwrap(),
            vec![Node::UnaryExpr {
                op: Operator::Plus,
                child: Box::new(Node::Int(1))
            }]
        );
        assert_eq!(format!("{}", plus_one.unwrap()[0]), "+1");

        let neg_two = parse("-2");
        assert!(neg_two.is_ok());
        assert_eq!(
            neg_two.clone().unwrap(),
            vec![Node::UnaryExpr {
                op: Operator::Min,
                child: Box::new(Node::Int(2))
            }]
        );
        assert_eq!(format!("{}", neg_two.unwrap()[0]), "-2");
    }
    #[test]
    fn binary_expr() {
        let sum = parse("1 + 2");
        assert!(sum.is_ok());
        assert_eq!(
            sum.clone().unwrap(),
            vec![Node::BinaryExpr {
                op: Operator::Plus,
                lhs: Box::new(Node::Int(1)),
                rhs: Box::new(Node::Int(2))
            }]
        );
        assert_eq!(format!("{}", sum.unwrap()[0]), "1 + 2");
        let minus = parse("1   -  \t  2");
        assert!(minus.is_ok());
        assert_eq!(
            minus.clone().unwrap(),
            vec![Node::BinaryExpr {
                op: Operator::Min,
                lhs: Box::new(Node::Int(1)),
                rhs: Box::new(Node::Int(2))
            }]
        );
        assert_eq!(format!("{}", minus.unwrap()[0]), "1 - 2");
        // fails as there's no rhs:
        // let paran_sum = parse("(1 + 2)");
        // assert!(paran_sum.is_ok());
    }

    #[test]
    fn nested_expr() {
        fn test_expr(expected: &str, src: &str) {
            assert_eq!(
                expected,
                parse(src)
                    .unwrap()
                    .iter()
                    .fold(String::new(), |acc, arg| acc + &format!("{}", arg))
            );
        }

        test_expr("1 + 2 + 3", "(1 + 2) + 3");
        test_expr("1 + 2 + 3", "1 + (2 + 3)");
        test_expr("1 + 2 + 3 + 4", "1 + (2 + (3 + 4))");
        test_expr("1 + 2 + 3 - 4", "(1 + 2) + (3 - 4)");
    }

    #[test]
    fn multiple_operators() {
        assert_eq!(
            parse("1+2+3").unwrap(),
            vec![Node::BinaryExpr {
                op: Operator::Plus,
                lhs: Box::new(Node::BinaryExpr {
                    op: Operator::Plus,
                    lhs: Box::new(Node::Int(1)),
                    rhs: Box::new(Node::Int(2)),
                }),
                rhs: Box::new(Node::Int(3)),
            }]
        )
    }

    #[test]
    fn negative_first_number() {
        // Issue #17: First number in expression cannot be negative
        let result = parse("-1 + 2");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![Node::BinaryExpr {
                op: Operator::Plus,
                lhs: Box::new(Node::UnaryExpr {
                    op: Operator::Min,
                    child: Box::new(Node::Int(1))
                }),
                rhs: Box::new(Node::Int(2))
            }]
        );

        // Also test -2 + 5 = 3
        let result = parse("-2 + 5");
        assert!(result.is_ok());
    }

    #[test]
    fn whitespace_handling() {
        // Issue #13: Parser should treat linefeed as whitespace
        let result = parse("1+2\n");
        assert!(result.is_ok());

        let result = parse("1 + 2\r\n");
        assert!(result.is_ok());
    }
}
