
use std::collections::HashMap;
use std::str::FromStr;

use crate::tokeniser::Token;
use crate::ast::{
    StatementNode, ExpressionNode, LookupNode, LetUnletNode,
    FractionNode, BinopNode, IfNode, ModopNode, FunctionNode,
    CatchNode, ArrayLiteralNode, Module, RefUnrefNode
};
use crate::interpreter::{Fraction, Instruction};


pub struct Parser {
    tokens: Vec<Token>,
    token_pos: usize,
    memo: HashMap<(usize, String), (usize, Parsed)>
}

type VecStatementNode = Vec<StatementNode>;
#[derive(Clone)]
pub enum Parsed {
    String(Option<String>),
    Token(Option<Token>),
    VecStatementNode(Option<VecStatementNode>),
    StatementNode(Option<StatementNode>),
    ExpressionNode(Option<ExpressionNode>),
    Instruction(Option<Instruction>),
    LookupNode(Option<LookupNode>),
    IfNode(Option<IfNode>),
    CatchNode(Option<CatchNode>),
    FunctionNode(Option<FunctionNode>),
    ArrayLiteralNode(Option<ArrayLiteralNode>)
}

macro_rules! memoise {
    ($raw_func:ident as $out_func:ident -> $ret_type:ident) => {
        fn $out_func(&mut self) -> Option<$ret_type> {
            let pos = self.mark();
            let key = (pos, String::from(stringify!($raw_func)));
            match self.memo.get(&key) {
                Some((end, result)) => {
                    let end = *end;
                    let result = (*result).clone();
                    if let Parsed::$ret_type(ret) = result {
                        self.reset(end);
                        return ret;
                    } else {unreachable!()}
                },
                None => {
                    let result = self.$raw_func();
                    let new_pos = self.mark();
                    let memo = Parsed::$ret_type(result.clone());
                    self.memo.insert(key, (new_pos, memo));
                    result
                }
            }
        }
    }
}

#[allow(unused_macros)]
macro_rules! memoise_recursive {
    ($raw_func:ident as $out_func:ident -> $ret_type:ident) => {
        fn $out_func(&mut self) -> Option<$ret_type> {
            let pos = self.mark();
            let key = (pos, String::from(stringify!($raw_func)));
            match self.memo.get(&key) {
                Some((end, result)) => {
                    let end = *end;
                    let result = (*result).clone();
                    if let Parsed::$ret_type(ret) = result {
                        self.reset(end);
                        return ret;
                    } else {unreachable!()}
                },
                None => {
                    let (mut lastres, mut lastpos) = (None, pos);
                    let memo = Parsed::$ret_type(lastres.clone());
                    self.memo.insert(key.clone(), (lastpos, memo));
                    loop {
                        self.reset(pos);
                        let result = self.$raw_func();
                        let endpos = self.mark();
                        if endpos <= lastpos {break};
                        lastres = result;
                        lastpos = endpos;
                        let memo = Parsed::$ret_type(lastres.clone());
                        self.memo.insert(key.clone(), (lastpos, memo));
                    }
                    self.reset(lastpos);
                    return lastres;
                }
            }
        }
    }
}


pub fn parse(tokens: Vec<Token>) -> Option<Module>{
    let mut parser = Parser{tokens, token_pos: 0, memo: HashMap::new()};
    if let Some(func) = parser.function() {
        Some(Module{functions: vec![func]})
    } else {
        None
    }
}


impl Parser {

    fn mark(&self) -> usize {
        self.token_pos
    }

    fn reset(&mut self, pos: usize) {
        self.token_pos = pos;
    }

    fn expect_literal(&mut self, value: &str) -> bool {
        if let Some(tokenref) =  self.tokens.get(self.token_pos).as_ref() {
            if tokenref.string_ == value {
                self.token_pos += 1;
                return true;
            };
        };
        false
    }

    fn expect_type(&mut self, type_: &str) -> Option<Token> {
        if let Some(tokenref) =  self.tokens.get(self.token_pos).as_ref() {
            if tokenref.type_ == type_ {
                self.token_pos += 1;
                return Some((*tokenref).clone());
            }
        }
        None
    }

    fn repeat<F, R>(&mut self, method: F, allow_empty: bool) -> Option<Vec<R>>
        where F: Copy + Fn(&mut Parser) -> Option<R>
    {
        let pos = self.mark();
        let mut results = Vec::new();
        loop {
            match method(self) {
                Some(result) => results.push(result),
                None => break
            };
        };
        if results.is_empty() && !allow_empty {
            self.reset(pos);
            None
        } else {
            Some(results)
        }
    }

    fn join<F, R>(&mut self, item_method: F, seperator: &str) -> Vec<R>
        where F: Copy + Fn(&mut Parser) -> Option<R>
    {
        let mut ret = Vec::new();
        match item_method(self) {
            Some(item) => ret.push(item),
            None => return ret
        }
        loop {
            let pos = self.mark();
            if !self.expect_literal(seperator) {return ret}
            match item_method(self) {
                Some(item) => ret.push(item),
                None => {
                    self.reset(pos);
                    return ret
                }
            }
        }
    }

    memoise!(function_ as function -> FunctionNode);
    pub fn function_(&mut self) -> Option<FunctionNode> {
        let pos = self.mark();

        if self.expect_literal("fn") {
        if let Some(name) = self.name() {
        if self.expect_literal("(") {
        let borrow_params = self.join(Parser::name, ",");
        if self.expect_literal(")") {
        if self.expect_literal("(") {
        let steal_params = self.join(Parser::name, ",");
        if self.expect_literal(")") {
        if self.expect_literal("{") {
        if let Some(stmts) = self.statements() {
        if self.expect_literal("}") {
        if self.expect_literal("~") {
        if self.name() == Some(name.clone()) {
        if self.expect_literal("(") {
        let return_params = self.join(Parser::name, ",");
        if self.expect_literal(")") {
            return Some(FunctionNode{
                name, borrow_params, steal_params, return_params, stmts
            });
        }}}}}}}}}}}}};

        self.reset(pos);
        None
    }

    memoise!(statements_ as statements -> VecStatementNode);
    pub fn statements_(&mut self) -> Option<Vec<StatementNode>> {
        self.repeat(Parser::statement, true)
    }

    memoise!(statement_ as statement -> StatementNode);
    pub fn statement_(&mut self) -> Option<StatementNode> {
        if let Some(stmt) = self.letunlet_stmt() {return Some(stmt);}
        if let Some(stmt) = self.refunref_stmt() {return Some(stmt);}
        if let Some(stmt) = self.modop_stmt() {return Some(stmt);}
        if let Some(stmt) = self.if_stmt() {return Some(stmt);}
        if let Some(stmt) = self.catch_stmt() {return Some(stmt);}
        None
    }

    memoise!(catch_stmt_ as catch_stmt -> StatementNode);
    pub fn catch_stmt_(&mut self) -> Option<StatementNode> {
        let pos = self.mark();

        if self.expect_literal("catch") {
        if self.expect_literal("(") {
        if let Some(expr) = self.expression() {
        if self.expect_literal(")") {
        if self.expect_literal(";") {
            return Some(StatementNode::Catch(Box::new(
                CatchNode{expr}
            )));
        }}}}};

        self.reset(pos);
        None
    }

    memoise!(if_stmt_ as if_stmt -> StatementNode);
    pub fn if_stmt_(&mut self) -> Option<StatementNode> {
        let pos = self.mark();

        if self.expect_literal("if") {
        if self.expect_literal("(") {
        if let Some(fwd_expr) = self.expression() {
        if self.expect_literal(")") {
        if self.expect_literal("{") {
        if let Some(if_stmts) = self.statements() {
        if self.expect_literal("}") {
        let else_stmts = self.else_block();
        if self.expect_literal("~") {
        if self.expect_literal("if") {
        if self.expect_literal("(") {
        let bkwd_expr = self.expression();
        if self.expect_literal(")") {
        if self.expect_literal(";") {
            let else_stmts = match else_stmts {
                Some(stmts) => stmts,
                None => Vec::new()
            };
            let bkwd_expr = match bkwd_expr {
                Some(expr) => expr,
                None => fwd_expr.clone()
            };
            return Some(StatementNode::If(Box::new(
                IfNode{fwd_expr, if_stmts, else_stmts, bkwd_expr}
            )));
        }}}}}}}}}}}};

        self.reset(pos);
        None
    }

    memoise!(else_block_ as else_block -> VecStatementNode);
    pub fn else_block_(&mut self) -> Option<Vec<StatementNode>> {
        let pos = self.mark();

        if self.expect_literal("else") {
        if self.expect_literal("{") {
        if let Some(stmts) = self.statements() {
        if self.expect_literal("}") {
            return Some(stmts);
        }}}};
        self.reset(pos);

        None
    }


    memoise!(modop_stmt_ as modop_stmt -> StatementNode);
    pub fn modop_stmt_(&mut self) -> Option<StatementNode> {
        let pos = self.mark();

        if let Some(lookup) = self.lookup() {
        if let Some(op) = self.modop() {
        if let Some(rhs) = self.expression() {
        if self.expect_literal(";") {
            return Some(StatementNode::Modop(Box::new(
                ModopNode{lookup, op, rhs}
            )));
        }}}};

        self.reset(pos);
        None
    }

    memoise!(refunref_stmt_ as refunref_stmt -> StatementNode);
    pub fn refunref_stmt_(&mut self) -> Option<StatementNode> {
        let pos = self.mark();

        if let Some(name) = self.name() {
        if self.expect_literal(":=") {
        if self.expect_literal("&") {
        if let Some(rhs) = self.lookup() {
        if self.expect_literal(";") {
            return Some(StatementNode::RefUnref(Box::new(
                RefUnrefNode{name, rhs, is_unref: false}
            )));
        }}}}};
        self.reset(pos);

        if let Some(name) = self.name() {
        if self.expect_literal("=:") {
        if self.expect_literal("&") {
        if let Some(rhs) = self.lookup() {
        if self.expect_literal(";") {
            return Some(StatementNode::RefUnref(Box::new(
                RefUnrefNode{name, rhs, is_unref: true}
            )));
        }}}}};
        self.reset(pos);

        None
    }


    memoise!(letunlet_stmt_ as letunlet_stmt -> StatementNode);
    pub fn letunlet_stmt_(&mut self) -> Option<StatementNode> {
        let pos = self.mark();

        if let Some(name) = self.name() {
        if self.expect_literal(":=") {
        if let Some(rhs) = self.expression() {
        if self.expect_literal(";") {
            return Some(StatementNode::LetUnlet(Box::new(
                LetUnletNode{name, rhs, is_unlet: false}
            )));
        }}}};
        self.reset(pos);

        if let Some(name) = self.name() {
        if self.expect_literal("=:") {
        if let Some(rhs) = self.expression() {
        if self.expect_literal(";") {
            return Some(StatementNode::LetUnlet(Box::new(
                LetUnletNode{name, rhs, is_unlet: true}
            )));
        }}}};
        self.reset(pos);
     
        None
    }


    memoise_recursive!(expression_ as expression -> ExpressionNode);
    pub fn expression_(&mut self) -> Option<ExpressionNode> {
        let pos = self.mark();
        
        if let Some(token) = self.expect_type("NUMBER") {
            let value = Fraction::from_str(&token.string_[..]).unwrap();
            let value = FractionNode{value};
            return Some(ExpressionNode::Fraction(Box::new(value)));
        };
        
        if let Some(x) = self.array_literal() {
            return Some(ExpressionNode::ArrayLiteral(Box::new(x)));
        }
        
        if let Some(lhs) = self.expression() {
        if let Some(op)  = self.binop() {
        if let Some(rhs) = self.expression() {
            return Some(
                ExpressionNode::Binop(Box::new(
                    BinopNode{lhs, rhs, op}
            )));
        }}};
        self.reset(pos);

        if let Some(lookup) = self.lookup() {
            return Some(ExpressionNode::Lookup(Box::new(lookup)));
        };

        None
    }

    memoise!(array_literal_ as array_literal -> ArrayLiteralNode);
    pub fn array_literal_(&mut self) -> Option<ArrayLiteralNode> {
        let pos = self.mark();

        if self.expect_literal("[") {
        let items = self.join(Parser::expression, ",");
        if self.expect_literal("]") {
            return Some(ArrayLiteralNode{items});
        }}

        self.reset(pos);
        None
    }

    memoise!(binop_ as binop -> Instruction);
    pub fn binop_(&mut self) -> Option<Instruction> {
        if self.expect_literal("+") { return Some(Instruction::BinopAdd) };
        if self.expect_literal("-") { return Some(Instruction::BinopSub) };
        if self.expect_literal("*") { return Some(Instruction::BinopMul) };
        if self.expect_literal("/") { return Some(Instruction::BinopDiv) };
        None
    }


    memoise!(modop_ as modop -> Instruction);
    pub fn modop_(&mut self) -> Option<Instruction> {
        if self.expect_literal("+=") { return Some(Instruction::BinopAdd) };
        if self.expect_literal("-=") { return Some(Instruction::BinopSub) };
        if self.expect_literal("*=") { return Some(Instruction::BinopMul) };
        if self.expect_literal("/=") { return Some(Instruction::BinopDiv) };
        None
    }


    memoise!(lookup_ as lookup -> LookupNode);
    pub fn lookup_(&mut self) -> Option<LookupNode> {
        let pos = self.mark();

        if let Some(name)    = self.name() {
        if let Some(indices) = self.repeat(Parser::index, true) {
            return Some(LookupNode{name, indices});
        }};

        self.reset(pos);
        None
    }

    memoise!(index_ as index -> ExpressionNode);
    pub fn index_(&mut self) -> Option<ExpressionNode> {
        let pos = self.mark();

        if self.expect_literal("[") {
        if let Some(expr) = self.expression() {
        if self.expect_literal("]") {
            return Some(expr);
        }}};

        self.reset(pos);
        None
    }

    memoise!(name_ as name -> String);
    pub fn name_(&mut self) -> Option<String> {
        let pos = self.mark();

        let has_dot = self.expect_literal(".");
        if let Some(token) = self.expect_type("NAME") {
            return Some(
                if has_dot { String::from(".") + &token.string_ }
                else       { token.string_ }
            );
        };

        self.reset(pos);
        None
    }

}