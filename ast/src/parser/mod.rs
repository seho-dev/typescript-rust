use std::{error::Error, path::Path, sync::Arc};

use crate::ast::{
    function::Param,
    module::{Import, ImportAlias, Module},
    statement::Statement,
    trycatch::TryCatch,
    typedefinition::{TypeBlock, TypeDefinition},
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use self::{
    class::parse_class,
    expression::{parse_const, parse_expression, parse_let, parse_term},
    ifs::parse_if,
    interface::parse_interface,
    repeat::{parse_for, parse_for_in, parse_for_of},
    switch::parse_switch, function::{parse_function, parse_param_kind},
};

mod class;
mod expression;
mod function;
mod ifs;
mod interface;
mod repeat;
mod switch;

#[derive(Parser)]
#[grammar = "parser/typescript.pest"] // relative to src
pub struct TypeScriptParser;

fn parse_statement(stmnt: Pair<Rule>) -> Option<Statement> {
    let stmnt = stmnt.into_inner().next().unwrap();

    match stmnt.as_rule() {
        Rule::Const => Some(parse_const(stmnt)),
        Rule::Let => Some(parse_let(stmnt)),
        Rule::Assign => Some(Statement::Expression(Arc::new(parse_term(stmnt)))),
        Rule::If => {
            let _if = parse_if(stmnt);
            Some(Statement::If(_if))
        }
        Rule::Switch => {
            let switch = parse_switch(stmnt);
            Some(Statement::Switch(switch))
        }
        Rule::For => {
            let repeat = parse_for(stmnt);
            Some(Statement::Loop(repeat))
        }
        Rule::ForOf => {
            let repeat = parse_for_of(stmnt);
            Some(Statement::Loop(repeat))
        }
        Rule::ForIn => {
            let repeat = parse_for_in(stmnt);
            Some(Statement::Loop(repeat))
        }
        Rule::Function => {
            let func = parse_function(stmnt);
            Some(Statement::Function(func))
        }
        Rule::Return => Some(Statement::Return(parse_expression(
            stmnt.into_inner().next().unwrap(),
        ))),
        Rule::TryCatch => Some(Statement::TryCatch(parse_trycatch(stmnt))),
        Rule::Throw => Some(Statement::Throw(parse_expression(
            stmnt.into_inner().next().unwrap(),
        ))),
        _ => None,
    }
}

fn parse_statements(stmnt: Pair<Rule>) -> Vec<Statement> {
    let mut block = Vec::new();

    for stmnt in stmnt.into_inner() {
        if let Some(s) = parse_statement(stmnt) {
            block.push(s);
        }
    }

    block
}

fn parse_trycatch(stmnt: Pair<Rule>) -> TryCatch {
    let mut inner = stmnt.into_inner();

    let try_block = parse_statements(inner.next().unwrap());
    let catch_name = inner.next().unwrap().as_str().into();
    let catch_block = parse_statements(inner.next().unwrap());

    TryCatch {
        try_block,
        catch_name,
        catch_block,
    }
}

fn parse_type(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();

    let name = inner.next().unwrap().as_str();
    let mut blocks = Vec::new();
    let mut aggregates = Vec::new();

    let definitions = inner.next().unwrap().into_inner();
    for def in definitions {
        match def.as_rule() {
            Rule::TypeBlock => {
                let mut attributes = Vec::new();
                for tuple in def.into_inner() {
                    let mut inner = tuple.into_inner();

                    let name = inner.next().unwrap().as_str();
                    let kind = inner.next().unwrap();
                    attributes.push(Param {
                        name: name.to_string(),
                        kinds: parse_param_kind(kind),
                        default: None,
                    });
                }

                blocks.push(TypeBlock { attributes })
            }
            Rule::Name => {
                aggregates.push(def.as_str().to_string());
            }
            _ => {}
        }
    }

    Statement::Type(TypeDefinition {
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
