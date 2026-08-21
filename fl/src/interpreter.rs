use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use core::fmt;
use std::{collections::HashMap, ptr};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Unit, // ()
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr = ptr::from_ref(self) as usize;
        match self {
            Value::Int(n) => write!(f, "{n} "),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Function { params, .. } => {
                write!(f, "<function({})> at {addr:#x}", params.join(", "))
            }
            Value::Unit => write!(f, "()"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
struct Frame {
    locals: HashMap<String, Value>,
}

impl Frame {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter {
    globals: HashMap<String, Value>,
    call_stack: Vec<Frame>,
    trace: bool,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            globals: HashMap::new(),
            call_stack: vec![Frame::new()],
            trace: false,
        }
    }
}

enum ControlFlow {
    Continue(Value),
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        for stmt in program {
            self.exec_stmt(stmt)?;
        }

        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Function { name, params, body } => {
                self.globals.insert(
                    name.clone(),
                    Value::Function {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok(ControlFlow::Continue(Value::Unit))
            }

            Stmt::Return(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(ControlFlow::Return(value))
            }

            Stmt::Assignment { name, value } => {
                let val = self.eval_expr(value)?;
                self.current_frame_mut().locals.insert(name.clone(), val);
                Ok(ControlFlow::Continue(Value::Unit))
            }

            Stmt::Expr(expr) => {
                let value = self.eval_expr(expr)?;
                Ok(ControlFlow::Continue(value))
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        if self.trace {
            eprintln!("[trace] eval: {}", expr);
        }
        match expr {
            Expr::Print(args) => {
                for expr in args {
                    let value = self.eval_expr(expr)?;
                    println!("{}", value);
                }
                Ok(Value::Unit)
            }
            Expr::Int(n) => Ok(Value::Int(*n)),

            Expr::Bool(b) => Ok(Value::Bool(*b)),

            Expr::Var(ident) => self.lookup_var(ident),

            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr)?;
                match (op, val) {
                    (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    (op, val) => Err(format!("Cannot apply {:?} to {:?}", op, val)),
                }
            }

            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary_op(*op, l, r)
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.eval_expr(cond)?;
                if let Value::Bool(b) = cond {
                    let branch = if b { then_branch } else { else_branch };
                    let mut result = Value::Unit;
                    for stmt in branch {
                        match self.exec_stmt(stmt)? {
                            ControlFlow::Continue(value) => result = value,
                            ControlFlow::Return(value) => return Ok(value),
                        }
                    }
                    return Ok(result);
                }
                Err(format!("Condition must be boolean, got {:?}", cond))
            }

            Expr::While { cond, body } => {
                loop {
                    let cond_val = self.eval_expr(cond)?;
                    if let Value::Bool(b) = cond_val {
                        if !b {
                            break;
                        }
                        for stmt in body {
                            match self.exec_stmt(stmt)? {
                                ControlFlow::Continue(_) => {}
                                ControlFlow::Return(value) => return Ok(value),
                            }
                        }
                    } else {
                        return Err(format!(
                            "While condition must be boolean, got {:?}",
                            cond_val
                        ));
                    }
                }
                Ok(Value::Unit)
            }

            Expr::Block(stmts) => {
                let mut result = Value::Unit;
                for stmt in stmts {
                    match self.exec_stmt(stmt)? {
                        ControlFlow::Continue(value) => result = value,
                        ControlFlow::Return(value) => return Ok(value),
                    }
                }
                Ok(result)
            }

            Expr::Call { name, args } => {
                let func = self.lookup_var(name)?;
                let (params, body) = match func {
                    Value::Function { params, body } => (params, body),
                    _ => return Err(format!("{} is not a function", name)),
                };
                let args_val: Vec<Value> = args
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;

                if params.len() != args_val.len() {
                    return Err(format!(
                        "Function {} expected {} arguments, got {}",
                        name,
                        params.len(),
                        args_val.len()
                    ));
                }

                let mut frame = Frame::new();
                for (param, arg) in params.iter().zip(args_val) {
                    frame.locals.insert(param.clone(), arg);
                }
                self.call_stack.push(frame);
                let mut result = Value::Unit;
                for stmt in body {
                    match self.exec_stmt(&stmt)? {
                        ControlFlow::Continue(value) => result = value,
                        ControlFlow::Return(value) => {
                            self.call_stack.pop();
                            return Ok(value);
                        }
                    }
                }
                self.call_stack.pop();
                Ok(result)
            }

            Expr::For {
                var,
                start,
                end,
                body,
            } => {
                let start = self.eval_expr(start)?;
                let end = self.eval_expr(end)?;

                let ns;
                let ne;

                if let Value::Int(_ns) = start
                    && let Value::Int(_ne) = end
                {
                    ns = _ns;
                    ne = _ne;
                } else {
                    return Err(format!(
                        "start and end should both be ints, found {} and {}",
                        start, end
                    ));
                }

                for iteration_var in ns..ne {
                    let mut frame = Frame::new();
                    frame.locals.insert(var.clone(), Value::Int(iteration_var));
                    self.call_stack.push(frame);

                    for stmt in body {
                        match self.exec_stmt(stmt)? {
                            ControlFlow::Continue(_) => {}
                            ControlFlow::Return(value) => {
                                self.call_stack.pop();
                                return Ok(value);
                            }
                        }
                    }
                }

                self.call_stack.pop();
                Ok(Value::Unit)
            }
        }
    }

    fn lookup_var(&self, ident: &str) -> Result<Value, String> {
        if let Some(val) = self.current_frame().locals.get(ident) {
            return Ok(val.clone());
        }

        if let Some(val) = self.globals.get(ident) {
            return Ok(val.clone());
        }

        Err(format!(
            "Undefined variable not found in this scope: {}",
            ident
        ))
    }

    fn current_frame(&self) -> &Frame {
        self.call_stack
            .last()
            .expect("Call stack should never be empty")
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.call_stack
            .last_mut()
            .expect("Call stack should never be empty")
    }

    fn eval_binary_op(&self, op: BinaryOp, left: Value, right: Value) -> Result<Value, String> {
        match (op, &left, &right) {
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (BinaryOp::Mod, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Int(a % b))
                }
            }

            (BinaryOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (BinaryOp::Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Ne, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),

            (BinaryOp::Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Ne, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),

            _ => Err(format!(
                "Cannot apply {:?} to {:?} and {:?}",
                op, left, right
            )),
        }
    }
}
