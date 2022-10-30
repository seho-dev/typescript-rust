use std::collections::HashMap;

use pest::iterators::Pair;

use crate::ast::{
    function::{Function, Param},
    tstype::TsType,
};

use super::{parse_statement, Rule, expression::parse_term};

pub fn parse_param_kind(kind: Pair<Rule>) -> Vec<TsType> {
    let mut kinds = Vec::new();

    for k in kind.into_inner() {
        kinds.push(k.as_str().into());
    }

    kinds
}

pub fn parse_param(param: Pair<Rule>) -> Param {
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

pub fn parse_template_definition(tmp: Pair<Rule>) -> (String, Vec<TsType>) {
    let mut inner = tmp.into_inner();

    let name = inner.next().unwrap().as_str();

    let kinds = if let Some(typedef) = inner.next() {
        parse_param_kind(typedef)
    } else {
        Vec::new()
    };

    (name.into(), kinds)
}

pub fn parse_function(func: Pair<Rule>) -> Function {
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
