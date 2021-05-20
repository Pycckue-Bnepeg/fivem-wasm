use full_moon::ast::{
    Call, Expression, Field, FunctionArgs, FunctionCall, Prefix, Stmt, Suffix, Value,
};

#[derive(Debug)]
pub struct FuncExec {
    pub name: String,
    pub argument: Argument,
    pub opt_arg: Option<Argument>,
}

#[derive(Debug)]
pub enum Argument {
    String(String),
    Table(Vec<(String, Vec<Box<Argument>>)>),
}

impl Argument {
    pub fn to_string(&self) -> String {
        match self {
            Argument::String(str) => str.clone(),
            Argument::Table(_) => "table".to_owned(),
        }
    }
}

fn unwrap_name(call: &FunctionCall) -> Option<String> {
    match call.prefix() {
        Prefix::Name(name) => Some(name.token().to_string()),
        _ => None,
    }
}

fn unwrap_argument(suffix: Option<&Suffix>) -> Option<Argument> {
    let suffix = suffix?;

    match suffix {
        Suffix::Call(call) => match call {
            Call::AnonymousCall(args) => match args {
                FunctionArgs::String(named) => match named.token_type() {
                    full_moon::tokenizer::TokenType::StringLiteral { literal, .. } => {
                        Some(Argument::String(literal.to_string()))
                    }

                    _ => None,
                },

                FunctionArgs::TableConstructor(table) => {
                    let table = table
                        .fields()
                        .iter()
                        .filter_map(|field| match field {
                            Field::NoKey(expr) => match expr {
                                Expression::Value { value } => match &**value {
                                    Value::FunctionCall(call) => {
                                        let name = unwrap_name(call)?;
                                        let args = call
                                            .suffixes()
                                            .filter_map(|suf| unwrap_argument(Some(suf)))
                                            .map(|arg| Box::new(arg))
                                            .collect();

                                        Some((name, args))
                                    }

                                    Value::String(str) => match str.token_type() {
                                        full_moon::tokenizer::TokenType::StringLiteral {
                                            literal,
                                            ..
                                        } => Some((literal.to_string(), vec![])),

                                        _ => None,
                                    },

                                    _ => None,
                                },

                                _ => None,
                            },

                            _ => None,
                        })
                        .collect();

                    Some(Argument::Table(table))
                }

                _ => None,
            },

            _ => None,
        },

        _ => None,
    }
}

fn unwrap(stmt: &Stmt) -> Option<FuncExec> {
    match stmt {
        Stmt::FunctionCall(call) => {
            let mut suffixes = call.suffixes();
            let name = unwrap_name(call)?;
            let argument = unwrap_argument(suffixes.next())?;
            let opt_arg = unwrap_argument(suffixes.next());

            Some(FuncExec {
                name,
                argument,
                opt_arg,
            })
        }

        _ => None,
    }
}

pub fn parse_file(file: &str) -> Vec<FuncExec> {
    let types = std::fs::read_to_string(file).unwrap();
    let ast_types = full_moon::parse(&types).unwrap();

    ast_types
        .nodes()
        .stmts()
        .filter_map(|stmt| unwrap(stmt))
        .collect()
}
