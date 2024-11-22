use std::collections::HashMap;

use thiserror::Error;

use crate::pass::{FromNamedParsedValue, Pass, RenderPassError};

use super::{ast::{Expr, Statement}, cli::PassArg};

pub struct Interpreter {
    pub passes: HashMap<String, Box<dyn Pass>>,
    pub edges: HashMap<String, Vec<String>>,
    pub display: Option<String>,
    symbols: HashMap<String, ParsedValue>,
    args: HashMap<String, Expr>,
}

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("invalid pass '{0}'. expected struct, tuple struct, or unit struct")]
    InvalidPass(String),
    #[error("invalid pass assignment with left hand side '{0}', expected right hand side to be struct, tuple struct, or unit struct")]
    InvalidPassAssignment(String),
    #[error("multiple display calls")]
    MultipleDisplays,
    #[error("invalid type. expected {0} but found {1}")]
    InvalidType(String, String),
    #[error("required argument '{0}' was not provided")]
    MissingArgument(String),
    #[error(transparent)]
    RenderPass(#[from] RenderPassError),
}

#[derive(Debug, Clone)]
pub enum ParsedValue {
    Int(i32),
    Float(f32),
    Path(String),
    Bool(bool),
    UnitStruct(String),
    /// A struct with named fields or a tuple struct with fields named `0`, `1`, `2`, etc.
    Struct {
        name: String,
        fields: HashMap<String, Box<ParsedValue>>,
    },
}

impl ParsedValue {
    pub fn struct_name(&self) -> Option<String> {
        match self {
            ParsedValue::UnitStruct(name) => Some(name.to_string()),
            ParsedValue::Struct { name, .. } => Some(name.to_string()),
            _ => None
        }
    }

    pub fn struct_properties(self) -> Option<(String, HashMap<String, Box<ParsedValue>>)> {
        match self {
            ParsedValue::UnitStruct(name) => Some((name, HashMap::new())),
            ParsedValue::Struct { name, fields } => Some((name, fields)),
            _ => None
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            ParsedValue::Int(_) => "int".to_string(),
            ParsedValue::Float(_) => "float".to_string(),
            ParsedValue::Path(_) => "path".to_string(),
            ParsedValue::Bool(_) => "bool".to_string(),
            ParsedValue::UnitStruct(name) => format!("unit struct `{}`", name),
            ParsedValue::Struct { name, .. } => format!("struct `{}`", name)
        }
    }
}

impl Interpreter {
    pub fn new(args: Vec<PassArg>) -> Self {
        let mut args_map = HashMap::new();

        for arg in args {
            args_map.insert(arg.name, arg.value);
        }

        Self {
            passes: HashMap::new(),
            edges: HashMap::new(),
            display: None,
            symbols: HashMap::new(),
            args: args_map,
        }
    }

    #[allow(clippy::vec_box)]
    pub fn run(&mut self, stmts: Vec<Box<Statement>>) -> Result<(), InterpreterError> {
        for stmt in stmts {
            self.run_stmt(*stmt)?;
        }

        Ok(())
    }

    fn run_stmt(&mut self, stmt: Statement) -> Result<(), InterpreterError> {
        match stmt {
            Statement::Assign { var, value: expr } => {
                let value = self.run_expr(*expr)?;
                self.symbols.insert(var, value);
            },
            Statement::Pass { name, value: expr } => {
                let value = self.run_expr(*expr)?;

                let Some(pass_name) = value.struct_name() else {
                    return Err(InterpreterError::InvalidPassAssignment(name));
                };

                let pass = <Box<dyn Pass>>::from_named_parsed_value(&pass_name, value)?;

                self.passes.insert(name, pass);
            },
            Statement::Edge { pass, dependencies } => {
                self.edges.insert(pass, dependencies);
            },
            Statement::Display { pass } => {
                if self.display.is_some() {
                    return Err(InterpreterError::MultipleDisplays);
                };

                self.display = Some(pass);
            },
        }

        Ok(())
    }

    fn run_expr(&mut self, expr: Expr) -> Result<ParsedValue, InterpreterError> {
        match expr {
            Expr::Int(v) => Ok(ParsedValue::Int(v)),
            Expr::Float(v) => Ok(ParsedValue::Float(v)),
            Expr::Path(mut p) => {
                p.pop();
                p.remove(0);
                Ok(ParsedValue::Path(p))
            },
            Expr::VarAccess(var) => {
                match self.symbols.get(&var) {
                    Some(v) => Ok(v.clone()),
                    None => Err(InterpreterError::UndefinedVariable(var)),
                }
            },
            Expr::Ident(ident) => {
                match ident.as_str() {
                    "true" => Ok(ParsedValue::Bool(true)),
                    "false" => Ok(ParsedValue::Bool(false)),
                    _ => Ok(ParsedValue::UnitStruct(ident)),
                }
            },
            Expr::Argument { name, default } => {
                match self.args.get(&name) {
                    Some(expr) => {
                        self.run_expr(expr.clone())
                    },
                    None => {
                        if let Some(default) = default {
                            self.run_expr(*default)
                        } else {
                            Err(InterpreterError::MissingArgument(name))
                        }
                    },
                }
            },
            Expr::TupleStruct { name, fields } => {
                let mut field_values = HashMap::new();

                for (i, field_val) in fields.into_iter().enumerate() {
                    let value = self.run_expr(*field_val)?;
                    field_values.insert(i.to_string(), Box::new(value));
                }

                Ok(ParsedValue::Struct {
                    name,
                    fields: field_values,
                })
            },
            Expr::Struct { name, fields, update } => {
                let mut field_values = HashMap::new();

                for field in fields {
                    let value = self.run_expr(*field.value)?;
                    field_values.insert(field.ident, Box::new(value));
                }

                if let Some(update) = update {
                    let (update_name, update_fields) = match self.symbols.get(&update) {
                        Some(v) => v.clone().struct_properties()
                            .ok_or(InterpreterError::InvalidType(format!("struct `{}`", name), v.type_name()))?,
                        None => return Err(InterpreterError::UndefinedVariable(update)),
                    };

                    if update_name != name {
                        return Err(InterpreterError::InvalidType(format!("struct `{}`", name), format!("struct `{}`", update_name)))
                    }

                    for (update_field, update_value) in update_fields {
                        field_values.entry(update_field).or_insert(update_value);
                    }
                };

                Ok(ParsedValue::Struct {
                    name,
                    fields: field_values,
                })
            },
        }
    }
}
