use pest::iterators::Pair;

use crate::ast::repeat::Loop;

use super::{Rule, expression::{parse_expression, parse_let}, parse_statement};


pub fn parse_for(stmnt: Pair<Rule>) -> Loop {
    let mut inner = stmnt.into_inner();
    let init = vec![parse_let(inner.next().unwrap())];
    let cond = parse_expression(inner.next().unwrap());
    let after = parse_expression(inner.next().unwrap());
    let mut block = Vec::new();

    for st in inner.next().unwrap().into_inner() {
        if let Some(st) = parse_statement(st) {
            block.push(st);
        }
    }

    Loop::For {
        init,
        cond,
        after,
        block,
    }
}

pub fn parse_for_of(stmnt: Pair<Rule>) -> Loop {
    let mut inner = stmnt.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap());
    let mut block = Vec::new();

    for st in inner.next().unwrap().into_inner() {
        if let Some(st) = parse_statement(st) {
            block.push(st);
        }
    }

    Loop::ForOf { name, value, block }
}

pub fn parse_for_in(stmnt: Pair<Rule>) -> Loop {
    let mut inner = stmnt.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap());
    let mut block = Vec::new();

    for st in inner.next().unwrap().into_inner() {
        if let Some(st) = parse_statement(st) {
            block.push(st);
        }
    }

    Loop::ForIn { name, value, block }
}