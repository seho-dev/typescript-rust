use std::sync::Arc;

use pest::iterators::Pair;

use super::{parse_param_kind, Rule};
use crate::ast::{operation::AssignOperation, statement::Statement, tstype::TsType, value::Value};

fn parse_assign_definition(stmnt: Pair<Rule>) -> (String, Vec<TsType>) {
    let mut inner = stmnt.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let kinds = if let Some(inn) = inner.next() {
        parse_param_kind(inn)
    } else {
        Vec::new()
    };

    (name, kinds)
}

pub fn parse_const(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();
    let (name, _kinds) = parse_assign_definition(inner.next().unwrap());
    let expr = inner.next().unwrap();

    Statement::Const {
        name: name.as_str().to_string(),
        value: parse_expression(expr),
    }
}

pub fn parse_let(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();
    let (name, _kinds) = parse_assign_definition(inner.next().unwrap());
    let expr = inner.next().unwrap();

    Statement::Let {
        name: name.as_str().to_string(),
        value: parse_expression(expr),
    }
}

pub fn parse_assign(stmnt: Pair<Rule>) -> Value {
    let mut inner = stmnt.into_inner();
    let name = inner.next().unwrap();
    let op = inner.next().unwrap().as_str().into();
    let expr = inner.next().unwrap();

    Value::Assign {
        identifier: name.as_str().to_string(),
        op,
        value: parse_expression(expr),
    }
}

pub fn parse_call(stmnt: Pair<Rule>) -> Value {
    let mut inner = stmnt.into_inner();
    let identifier: Vec<String> = inner
        .next()
        .unwrap()
        .as_str()
        .split(".")
        .map(|n| n.to_string())
        .collect();

    let args = inner
        .next()
        .unwrap()
        .into_inner()
        .map(|n| parse_expression(n))
        .collect();

    Value::Call { identifier, args }
}

pub fn parse_term(term: Pair<Rule>) -> Value {
    match term.as_rule() {
        Rule::PostTerm => {
            let mut inner = term.into_inner();
            let name = inner.next().unwrap();
            let op = inner.next().unwrap();

            let op = match op.as_rule() {
                Rule::Inc => AssignOperation::Add,
                Rule::Dec => AssignOperation::Sub,
                _ => AssignOperation::Neutral,
            };

            Value::Assign {
                identifier: name.as_str().to_string(),
                op,
                value: Arc::new(Value::Number(1.0)),
            }
        }
        Rule::PrefixTerm => {
            let mut inner = term.into_inner();
            let op = inner.next().unwrap();
            let name = inner.next().unwrap();

            let op = match op.as_rule() {
                Rule::Inc => AssignOperation::Add,
                Rule::Dec => AssignOperation::Sub,
                _ => AssignOperation::Neutral,
            };

            Value::Assign {
                identifier: name.as_str().to_string(),
                op,
                value: Arc::new(Value::Number(1.0)),
            }
        }
        Rule::Call => parse_call(term),
        Rule::Assign => parse_assign(term),
        Rule::CaseTerm | Rule::Term => {
            let inner = term.into_inner().next().unwrap();

            match inner.as_rule() {
                Rule::Number => {
                    if let Ok(flt) = inner.as_str().parse::<f64>() {
                        Value::Number(flt)
                    } else {
                        Value::Undefined
                    }
                }
                Rule::Boolean => Value::Boolean(inner.as_str() == "true"),
                Rule::Identifier => {
                    let names: Vec<String> =
                        inner.as_str().split(".").map(|n| n.to_string()).collect();

                    Value::Identifier(names)
                }
                Rule::String => {
                    let data = inner.as_str();
                    Value::String(data[1..data.len() - 1].into())
                }
                Rule::Array => {
                    let mut array = Vec::new();

                    for stmnt in inner.into_inner() {
                        array.push(parse_expression(stmnt));
                    }

                    Value::Array(array)
                }
                _ => Value::Undefined,
            }
        }
        _ => Value::Undefined,
    }
}

pub fn parse_expression(expr: Pair<Rule>) -> Arc<Value> {
    let mut inner = expr.into_inner();

    let term = inner.next().unwrap();

    if let Some(op) = inner.next() {
        let term1 = inner.next().unwrap();
        Arc::new(Value::Expression {
            left: Arc::new(parse_term(term)),
            op: op.as_str().into(),
            right: Arc::new(parse_term(term1)),
        })
    } else {
        Arc::new(parse_term(term))
    }
}
