use std::sync::Arc;

use pest::iterators::Pair;

use crate::ast::switch::{Switch, Case};

use super::{Rule, expression::{parse_expression, parse_term}, parse_statement};

pub fn parse_switch(stmnt: Pair<Rule>) -> Switch {
    let mut inner = stmnt.into_inner();

    let value = parse_expression(inner.next().unwrap());
    let mut branches = Vec::new();
    let mut default = None;

    while let Some(stmnt) = inner.next() {
        match stmnt.as_rule() {
            Rule::Case => {
                let mut inner = stmnt.into_inner();
                let expr = Arc::new(parse_term(inner.next().unwrap()));
                let mut block = Vec::new();

                while let Some(stmnt) = inner.next() {
                    match stmnt.as_rule() {
                        Rule::Statement => {
                            if let Some(stmnt) = parse_statement(stmnt) {
                                block.push(stmnt);
                            }
                        }
                        Rule::Break => break,
                        _ => {}
                    }
                }

                branches.push(Case { expr, block });
            }
            Rule::Default => {
                let mut block = Vec::new();

                for inner in stmnt.into_inner() {
                    match inner.as_rule() {
                        Rule::Statement => {
                            if let Some(stmnt) = parse_statement(inner) {
                                block.push(stmnt);
                            }
                        }
                        Rule::Break => break,
                        _ => {
                            log::error!("unknown switch default statemnt: {:?}", inner);
                        }
                    }
                }

                default = Some(block);
            }
            _ => {
                log::error!("could not parse: {:?}", stmnt);
            }
        }
    }

    Switch {
        value,
        branches,
        default,
    }
}