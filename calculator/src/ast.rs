use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operator {
    Plus,
    Min,
    Mul,
    Div,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Operator::Plus => write!(f, "+"),
            Operator::Min => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Int(i32),
    Float(f32),
    UnaryExpr {
        op: Operator,
        child: Box<Node>,
    },
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Node::Int(n) => write!(f, "{}", n),
            Node::Float(n) => write!(f, "{}", n),
            Node::UnaryExpr { op, child } => write!(f, "{}{}", op, child),
            Node::BinaryExpr { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
        }
    }
}
