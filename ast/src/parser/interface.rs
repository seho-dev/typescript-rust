use pest::iterators::Pair;


use crate::ast::{statement::Statement, interface::Interface};

use super::{Rule, parse_function, function::parse_param};

pub fn parse_interface(stmnt: Pair<Rule>) -> Statement {
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