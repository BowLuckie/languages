use crate::{
    ast::{Node, Operator},
    compile::Compile,
};

pub struct Interpreter;

impl Interpreter {
    fn eval(node: &Node) -> i32 {
        match node {
            Node::Int(n) => *n,
            Node::UnaryExpr { op, child } => {
                let child = Self::eval(child);
                match op {
                    Operator::Plus => child,
                    Operator::Min => -child,
                    Operator::Mul => todo!(),
                    Operator::Div => todo!(),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let lhs_ret = Self::eval(lhs);
                let rhs_ret = Self::eval(rhs);

                match op {
                    Operator::Plus => lhs_ret + rhs_ret,
                    Operator::Min => lhs_ret - rhs_ret,
                    Operator::Mul => todo!(),
                    Operator::Div => todo!(),
                }
            }
        }
    }
}

impl Compile for Interpreter {
    type Output = Vec<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut ret;
        let mut out = vec![];
        for node in ast {
            ret = 0;
            ret += Self::eval(&node);
            out.push(ret);
        }
        out
    }
}
