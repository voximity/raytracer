use crate::{
    ast::NodeKind,
    interpret::{InterpretError, Interpreter, Value},
};

/// A function, callable from within the SDL.
pub struct Function {
    pub names: &'static [&'static str],
    pub arg_types: &'static [NodeKind],
    pub func: Box<dyn Send + Sync + Fn(&Interpreter, Vec<Value>) -> Result<Value, InterpretError>>,
}

impl Function {
    /// Instantiate a new function.
    pub fn new<F>(names: &'static [&'static str], arg_types: &'static [NodeKind], f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&Interpreter, Vec<Value>) -> Result<Value, InterpretError>,
    {
        Self {
            names,
            arg_types,
            func: Box::new(f),
        }
    }

    /// Try to evaluate with this function. Returns `None` if arguments don't match.
    pub fn try_eval(
        &self,
        interpreter: &Interpreter,
        args: Vec<Value>,
    ) -> Option<Result<Value, InterpretError>> {
        if args.iter().zip(self.arg_types.iter()).any(|(a, b)| a != b) {
            return None;
        }

        Some((self.func)(interpreter, args))
    }
}
