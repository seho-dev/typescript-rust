use std::{error::Error, path::Path, sync::Arc};

use crate::ast::{
    module::{ImportAlias, Module, Import},
    statement::{ElseIf, Param, Statement, ParamType},
    value::Value, tstype::Type,
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/typescript.pest"] // relative to src
pub struct TypeScriptParser;

fn parse_term(term: Pair<Rule>) -> Value {
    let inner = term.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::Number => {
            if let Ok(flt) = inner.as_str().parse::<f64>() {
                Value::Number(flt)
            } else {
                Value::Undefined
            }
        }
        Rule::Identifier => {
            let names: Vec<String> = inner.as_str().split(".").map(|n| n.to_string()).collect();

            Value::Identifier(names)
        }
        Rule::Call => {
            Value::Undefined
        }
        _ => Value::Undefined,
    }
}

fn parse_value(expr: Pair<Rule>) -> Arc<Value> {
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

fn parse_param_kind(kind: Pair<Rule>) -> Vec<ParamType> {
    let mut kinds = Vec::new();

    for k in kind.into_inner() {
        kinds.push(k.as_str().into());
    }

    kinds
}

fn parse_statement(stmnt: Pair<Rule>) -> Option<Statement> {
    let stmnt = stmnt.into_inner().next().unwrap();

    match stmnt.as_rule() {
        Rule::Const => {
            let mut inner = stmnt.into_inner();
            let name = inner.next().unwrap();
            let expr = inner.next().unwrap();

            Some(Statement::Const {
                name: name.as_str().to_string(),
                value: parse_value(expr),
            })
        }
        Rule::Let => {
            let mut inner = stmnt.into_inner();
            let name = inner.next().unwrap();
            let expr = inner.next().unwrap();

            Some(Statement::Let {
                name: name.as_str().to_string(),
                value: parse_value(expr),
            })
        }
        Rule::Assign => {
            let mut inner = stmnt.into_inner();
            let name = inner.next().unwrap();
            let expr = inner.next().unwrap();

            Some(Statement::Assign {
                identifier: name.as_str().to_string(),
                value: parse_value(expr),
            })
        }
        Rule::If => {
            let mut inner = stmnt.into_inner();
            let expr = inner.next().unwrap();
            let block = inner.next().unwrap();

            let mut block_stmnts = Vec::new();
            for stmnt in block.into_inner() {
                if let Some(s) = parse_statement(stmnt) {
                    block_stmnts.push(s);
                }
            }

            let mut else_block = Vec::new();
            let mut elseifs = Vec::new();
            for next in inner {
                match next.as_rule() {
                    Rule::ElseIf => {
                        let mut inner = next.into_inner();
                        let expr = inner.next().unwrap();
                        let block = inner.next().unwrap();

                        let mut block_statements = Vec::new();
                        for stmnt in block.into_inner() {
                            if let Some(s) = parse_statement(stmnt) {
                                block_statements.push(s);
                            }
                        }

                        elseifs.push(ElseIf {
                            expr: parse_value(expr),
                            block: block_statements,
                        });
                    }
                    Rule::Else => {
                        let block = next.into_inner().next().unwrap();
                        for stmnt in block.into_inner() {
                            if let Some(s) = parse_statement(stmnt) {
                                else_block.push(s);
                            }
                        }
                    }
                    _ => {}
                }
            }

            Some(Statement::If {
                expr: parse_value(expr),
                block: block_stmnts,
                elseifs,
                els: else_block,
            })
        }
        Rule::Function => {
            let mut inner = stmnt.into_inner();
            let name = inner.next().unwrap();
            let param_list = inner.next().unwrap();
            let block = inner.next().unwrap();
            let mut params = Vec::new();

            for p in param_list.into_inner() {
                let mut inner = p.into_inner();
                let name = inner.next().unwrap();

                params.push(Param {
                    name: name.as_str().to_string(),
                    kinds: parse_param_kind(inner.next().unwrap()),
                });
            }

            let mut block_statements = Vec::new();
            for stmnt in block.into_inner() {
                if let Some(s) = parse_statement(stmnt) {
                    block_statements.push(s);
                }
            }

            Some(Statement::Function {
                name: name.as_str().to_string(),
                params,
                block: block_statements,
            })
        }
        Rule::Call => {
            let mut inner = stmnt.into_inner();
            let identifier: Vec<String> = inner
                .next()
                .unwrap()
                .as_str()
                .split(".")
                .map(|n| n.to_string())
                .collect();

            let params = inner
                .next()
                .unwrap()
                .into_inner()
                .map(|n| parse_value(n))
                .collect();

            Some(Statement::Call { 
                identifier,
                params,
            })
        }
        _ => None,
    }
}

fn parse_interface(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();

    let name = inner.next().unwrap().as_str();
    Statement::Interface { name: name.to_string() }
}

fn parse_class(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();

    let name = inner.next().unwrap().as_str();
    Statement::Class { name: name.to_string() }
}

fn parse_type(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();

    let name = inner.next().unwrap().as_str();
    let mut blocks = Vec::new();
    let mut aggregates = Vec::new();

    let mut attributes = Vec::new();
    let definitions = inner.next().unwrap().into_inner();
    for def in definitions {
        match def.as_rule() {
            Rule::TypeBlock => {
                let tuple = def.into_inner().next().unwrap();
                let mut inner = tuple.into_inner();
                let name = inner.next().unwrap().as_str();
                let kind = inner.next().unwrap();
                attributes.push(Param { 
                    name: name.to_string(),
                    kinds: parse_param_kind(kind), 
                });
            }
            Rule::Name => {
                aggregates.push(def.as_str().to_string());
            }
            _ => {},
        }
    }

    Statement::Type(Type { 
        name: name.to_string(),
        blocks,
        aggregates,
     })
}

pub fn file<T: AsRef<Path>>(filename: T) -> Result<Module, Box<dyn Error>> {
    let src = std::fs::read_to_string(filename)?;
    source(&src)
}

pub fn source(source: &str) -> Result<Module, Box<dyn Error>> {
    let mut module = Module::new();
    let rules = TypeScriptParser::parse(Rule::Statements, &source)?
        .next()
        .unwrap();

    for stmnt in rules.into_inner() {
        match stmnt.as_rule() {
            Rule::Import => {
                let path = stmnt.into_inner().next().unwrap();
                let name = path.as_str();
                let clean_name = &name[1..name.len() - 1];

                module.imports.push(Import::Normal {
                    path: clean_name.to_string(),
                });
            }
            Rule::ImportFrom => {
                let mut inner = stmnt.into_inner();
                let namelist = inner.next().unwrap();
                let mut names = Vec::new();

                for n in namelist.into_inner() {
                    let mut inner = n.into_inner();
                    let name = inner.next().unwrap();
                    let name_str = name.as_str();

                    if let Some(alias) = inner.next() {
                        let alias_str = alias.as_str();
                        names.push(ImportAlias::Alias {
                            name: name_str.to_string(),
                            alias: alias_str.to_string(),
                        });
                    } else {
                        names.push(ImportAlias::None {
                            name: name_str.to_string(),
                        });
                    }
                }

                let file = inner.next().unwrap();
                let filename = file.as_str();

                module.imports.push(Import::From {
                    names,
                    path: filename[1..filename.len() - 1].to_string(),
                })
            }
            Rule::Interface => {
                module.statements.push(parse_interface(stmnt));
            }
            Rule::Class => {
                module.statements.push(parse_class(stmnt));
            }
            Rule::Type => {
                module.statements.push(parse_type(stmnt));
            }
            Rule::Statement => {
                if let Some(s) = parse_statement(stmnt) {
                    module.statements.push(s);
                }
            }
            _ => {}
        }
    }

    Ok(module)
}
