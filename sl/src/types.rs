use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type (64-bit signed)
    Int,
    /// Boolean type
    Bool,
    /// Function type: (param_types) -> return_type
    Function { params: Vec<Type>, ret: Box<Type> },
    /// Unit type (for statements with no value)
    Unit,
    /// Unknown type (for type inference)
    Unknown,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::Unknown => write!(f, "?"),
            Type::Function { params, ret } => {
                let params_str: Vec<_> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "({}) -> {}", params_str.join(", "), ret)
            }
        }
    }
}
