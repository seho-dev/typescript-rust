use std::collections::HashMap;

use pest::iterators::Pair;

use crate::ast::{class::Class, statement::Statement};

use super::{parse_function, Rule, function::{parse_template_definition, parse_param}};

pub fn parse_class(stmnt: Pair<Rule>) -> Statement {
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
