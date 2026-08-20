use core::fmt;

pub type Program = Vec<Stmt>;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Expr),
    Assignment {
        name: String,
        value: Expr,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Var(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        cond: Box<Expr>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Function { name, params, body } => {
                write!(f, "def {}({}) {{ ", name, params.join(", "))?;
                for stmt in body {
                    write!(f, "{} ", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::Return(expr) => write!(f, "return {}", expr),
            Stmt::Assignment { name, value } => write!(f, "{} = {}", name, value),
            Stmt::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(n) => write!(f, "{}", n),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Unary { op, expr } => write!(f, "({}{})", op, expr),
            Expr::Binary { op, left, right } => write!(f, "({} {} {})", left, op, right),
            Expr::Call { name, args } => {
                let args_str: Vec<_> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", name, args_str.join(", "))
            }
            Expr::If { cond, .. } => {
                write!(f, "if ({}) {{ ... }} else {{ ... }}", cond)
            }
            Expr::While { cond, .. } => {
                write!(f, "while ({}) {{ ... }}", cond)
            }
            Expr::Block(_) => write!(f, "{{ ... }}"),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
        }
    }
}
