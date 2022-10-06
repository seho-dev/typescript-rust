use std::{error::Error, path::Path, sync::Arc, collections::HashMap};

use crate::ast::{
    class::Class,
    function::{Function, Param},
    interface::Interface,
    module::{Import, ImportAlias, Module},
    statement::{ElseIf, Statement},
    tstype::TsType,
    typedefinition::{TypeBlock, TypeDefinition},
    value::Value,
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
        Rule::False => Value::Boolean(false),
        Rule::True => Value::Boolean(true),
        Rule::Identifier => {
            let names: Vec<String> = inner.as_str().split(".").map(|n| n.to_string()).collect();

            Value::Identifier(names)
        }
        Rule::String => {
            let data = inner.as_str();
            Value::String(data[1..data.len() - 1].into())
        }
        Rule::Call => {
            parse_call(inner)
        }
        _ => {
            Value::Undefined
        }
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

fn parse_param_kind(kind: Pair<Rule>) -> Vec<TsType> {
    let mut kinds = Vec::new();

    for k in kind.into_inner() {
        kinds.push(k.as_str().into());
    }

    kinds
}

fn parse_param(param: Pair<Rule>) -> Param {
    let mut inner = param.into_inner();
    let name = inner.next().unwrap();
    let mut kinds = Vec::new();
    let mut default = None;

    while let Some(t) = inner.next() {
        match t.as_rule() {
            Rule::TypeIdentifiers => {
                kinds = parse_param_kind(t);
            }
            Rule::Term => {
                default = Some(parse_term(t));
            }
            _ => {}
        }
    }

    Param {
        name: name.as_str().to_string(),
        kinds,
        default,
    }
}

fn parse_template_definition(tmp: Pair<Rule>) -> (String, Vec<TsType>) {
    let mut inner = tmp.into_inner();

    let name = inner.next().unwrap().as_str();

    let kinds = if let Some(typedef) = inner.next() {
        parse_param_kind(typedef)
    }
    else {
        Vec::new()
    };

    (name.into(), kinds)
}

fn parse_function(func: Pair<Rule>) -> Function {
    let mut name = None;
    let mut params = Vec::new();
    let mut returns = Vec::new();
    let mut block_statements = Vec::new();
    let mut template_args = HashMap::new();

    for inner in func.into_inner() {
        match inner.as_rule() {
            Rule::Name => {
                name = Some(inner.as_str().into());
            }
            Rule::TemplateDefinition => {
                for p in inner.into_inner() {
                    let (name, kinds) = parse_template_definition(p);
                    template_args.insert(name, kinds);
                }
            }
            Rule::FunctionDefinition => {
                for p in inner.into_inner() {
                    match p.as_rule() {
                        Rule::ParamList => {
                            for p in p.into_inner() {
                                params.push(parse_param(p));
                            }
                        }
                        Rule::ReturnType => {
                            for r in p.into_inner() {
                                returns.push(r.as_str().into());
                            }
                        }
                        _ => {}
                    }
                }
            }
            Rule::Block => {
                for stmnt in inner.into_inner() {
                    if let Some(s) = parse_statement(stmnt) {
                        block_statements.push(s);
                    }
                }
            }
            _ => {}
        }
    }

    let f = Function {
        name,
        is_async: false,
        template_args,
        params,
        returns,
        block: block_statements,
    };
    f
}

fn parse_call(stmnt: Pair<Rule>) -> Value {
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
        .map(|n| parse_value(n))
        .collect();

    Value::Call { identifier, args }
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
            let func = parse_function(stmnt);
            Some(Statement::Function(func))
        }
        Rule::Call => Some(Statement::Call(Arc::new(parse_call(stmnt)))),
        Rule::Return => {
            Some(Statement::Return(parse_value(stmnt.into_inner().next().unwrap())))
        }
        _ => None,
    }
}

fn parse_interface(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();

    let name = inner.next().unwrap().as_str();
    let mut extends = None;
    let mut attributes = Vec::new();
    let mut methods = Vec::new();

    while let Some(block) = inner.next() {
        match block.as_rule() {
            Rule::InterfaceExtends => {
                let inner = block.into_inner().next().unwrap();
                extends = Some(inner.as_str().into());
            }
            Rule::InterfaceBody => {
                for part in block.into_inner() {
                    match part.as_rule() {
                        Rule::InterfaceMethod => {
                            methods.push(parse_function(part));
                        }
                        Rule::InterfaceAttribute => {
                            attributes.push(parse_param(part.into_inner().next().unwrap()));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let interface = Interface {
        name: name.to_string(),
        extends,
        attributes,
        methods,
    };
    Statement::Interface(interface)
}

fn parse_class(stmnt: Pair<Rule>) -> Statement {
    let mut inner = stmnt.into_inner();

    let name = inner.next().unwrap().as_str();
    let mut extends = None;
    let mut implements = Vec::new();
    let mut attributes = Vec::new();
    let mut methods = Vec::new();
    let mut template_args = HashMap::new();

    while let Some(block) = inner.next() {
        match block.as_rule() {
            Rule::TemplateDefinition => {
                for inner in block.into_inner() {
                    let (name, args) = parse_template_definition(inner);
                    template_args.insert(name, args);
                }
            }
            Rule::Extends => {
                let inner = block.into_inner().next().unwrap();
                extends = Some(inner.as_str().into());
            }
            Rule::Implements => {
                for name in block.into_inner() {
                    implements.push(name.as_str().into());
                }
            }
            Rule::ClassBody => {
                for part in block.into_inner() {
                    match part.as_rule() {
                        Rule::Method => {
                            methods.push(parse_function(part));
                        }
                        Rule::ClassAttribute => {
                            attributes.push(parse_param(part.into_inner().next().unwrap()));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let class = Class {
        name: name.to_string(),
        extends,
        implements,
        attributes,
        methods,
        template_args,
    };
    Statement::Class(class)
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
