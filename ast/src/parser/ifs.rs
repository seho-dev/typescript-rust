use pest::iterators::Pair;

use crate::ast::ifelse::{ElseIf, IfElse};

use super::{expression::parse_expression, parse_statement, parse_statements, Rule};

pub fn parse_if(stmnt: Pair<Rule>) -> IfElse {
    let mut inner = stmnt.into_inner();
    let expr = inner.next().unwrap();
    let block = inner.next().unwrap();

    let block_stmnts = parse_statements(block);

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
                    expr: parse_expression(expr),
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

    IfElse {
        expr: parse_expression(expr),
        block: block_stmnts,
        elseifs,
        els: else_block,
    }
}
