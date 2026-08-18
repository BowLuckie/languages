use crate::ast::Node;

pub enum Ty {
    Int,
    Float,
}

pub fn infer_type(ast: &Vec<Node>) -> Ty {
    for node in ast {
        if recurse(node) {
            return Ty::Float;
        }
    }

    Ty::Int
}

fn recurse(node: &Node) -> bool {
    match node {
        Node::Int(_) => false,
        Node::Float(_) => true,
        Node::UnaryExpr { op: _, child } => recurse(child),
        Node::BinaryExpr { op: _, lhs, rhs } => recurse(lhs) || recurse(rhs),
    }
}
