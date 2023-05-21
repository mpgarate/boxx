use expr::Expr;
use parser::parser_error::ParserError;
use std::error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    SteppingOnValue(Expr),
    UnexpectedExpr(String, Expr),
    VariableNotFound(String),
    InvalidConstAssignment(Expr, String),
    InvalidTypeConversion(String, Expr),
    InvalidMemoryState(String),
    TooManyIterations(usize),
    ParserError(ParserError),
    IntegerOverflow,
    IntegerUnderflow,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RuntimeError::SteppingOnValue(ref e) => write!(f, "Stepping on a value {:?}", e),
            RuntimeError::UnexpectedExpr(ref s, ref e) => {
                write!(f, "Unexpected expression. Expected {} and found {:?}", s, e)
            }
            RuntimeError::VariableNotFound(ref e) => {
                write!(f, "Variable {:?} does not exist in memory", e)
            }
            RuntimeError::InvalidConstAssignment(ref e, ref s) => {
                write!(f, "Cannot assign {:?} to const {}", e, s)
            }
            RuntimeError::InvalidTypeConversion(ref s, ref e) => write!(
                f,
                "Invalid type conversion. Expected {} and found {:?}",
                s, e
            ),
            RuntimeError::InvalidMemoryState(ref s) => {
                write!(f, "Unexpected internal memory state: {}", s)
            }
            RuntimeError::TooManyIterations(ref n) => {
                write!(f, "Too many iterations while evaluating expression: {}", n)
            }
            RuntimeError::ParserError(ref err) => write!(f, "Parser error: {}", err),
            RuntimeError::IntegerOverflow => write!(
                f,
                "Integer overflow: Integer underflow: Value grew too large"
            ),
            RuntimeError::IntegerUnderflow => write!(f, "Integer underflow: Value grew too small"),
        }
    }
}

impl error::Error for RuntimeError {
    fn description(&self) -> &str {
        match *self {
            RuntimeError::SteppingOnValue(_) => "Stepping on a value",
            RuntimeError::UnexpectedExpr(_, _) => "Unexpected expression",
            RuntimeError::VariableNotFound(_) => "Variable does not exist in memory",
            RuntimeError::InvalidConstAssignment(_, _) => "Cannot assign to const",
            RuntimeError::InvalidTypeConversion(_, _) => "Invalid type conversion",
            RuntimeError::InvalidMemoryState(_) => "Unexpected internal memory state",
            RuntimeError::TooManyIterations(_) => "Too many iterations: {}",
            RuntimeError::ParserError(ref err) => &err.description(),
            RuntimeError::IntegerOverflow => "Integer overflow: Value grew too large",
            RuntimeError::IntegerUnderflow => "Integer underflow: Value grew too small",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            RuntimeError::SteppingOnValue(_) => None,
            RuntimeError::UnexpectedExpr(_, _) => None,
            RuntimeError::VariableNotFound(_) => None,
            RuntimeError::InvalidConstAssignment(_, _) => None,
            RuntimeError::InvalidTypeConversion(_, _) => None,
            RuntimeError::InvalidMemoryState(_) => None,
            RuntimeError::TooManyIterations(_) => None,
            RuntimeError::ParserError(ref err) => Some(err),
            RuntimeError::IntegerOverflow => None,
            RuntimeError::IntegerUnderflow => None,
        }
    }
}

impl From<ParserError> for RuntimeError {
    fn from(err: ParserError) -> RuntimeError {
        RuntimeError::ParserError(err)
    }
}
