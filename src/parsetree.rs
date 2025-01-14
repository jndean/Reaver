
use std::fmt;

use crate::interpreter;
use crate::syntaxchecker;
use crate::syntaxtree as ST;



pub trait Expression: fmt::Debug + ExpressionClone {

    fn to_syntax_node(self: Box<Self>,  ctx: &mut syntaxchecker::SyntaxContext) 
        -> Result<Box<dyn ST::Expression>, syntaxchecker::SyntaxError>;

    fn get_src_pos(&self) 
        -> (usize, usize);
}

pub type ExpressionNode = Box<dyn Expression>;

// This 'inbetween' trait allows us to implement Clone on Expression trait objects
pub trait ExpressionClone {
    fn clone_box(&self) ->ExpressionNode;
}
impl<T: 'static + Expression + Clone> ExpressionClone for T {
    fn clone_box(&self) -> ExpressionNode {
        Box::new(self.clone())
    }
}
impl Clone for ExpressionNode {
    fn clone(&self) -> ExpressionNode {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct FractionNode {
    pub line: usize,
    pub col: usize,
    pub value: interpreter::Fraction
}

impl fmt::Debug for FractionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct StringNode {
    pub line: usize,
    pub col: usize,
    pub value: String
}

#[derive(Clone, Debug)]
pub struct ArrayLiteralNode {
    pub line: usize,
    pub col: usize,
    pub items: Vec<ExpressionNode>
}

#[derive(Clone, Debug)]
pub struct ArrayRepeatNode {
    pub line: usize,
    pub col: usize,
    pub item: ExpressionNode,
    pub dimensions: ExpressionNode
}

#[derive(Clone, Debug)]
pub struct LookupNode {
    pub line: usize,
    pub col: usize,
    pub name: String,
    pub indices: Vec<ExpressionNode>
}

#[derive(Clone, Debug)]
pub struct BinopNode {
    pub lhs: ExpressionNode,
    pub rhs: ExpressionNode,
    pub op: interpreter::Instruction
}

#[derive(Clone, Debug)]
pub struct UniopNode {
    pub line: usize,
    pub col: usize,
    pub expr: ExpressionNode,
    pub op: interpreter::Instruction
}


pub trait Statement: fmt::Debug + StatementClone {
    fn to_syntax_node(
        self: Box<Self>,
        ctx: &mut syntaxchecker::SyntaxContext
    ) -> Result<Box<dyn ST::Statement>, syntaxchecker::SyntaxError>;
}

pub type StatementNode = Box<dyn Statement>;

// This 'inbetween' trait allows us to implement Clone on Statement trait objects
pub trait StatementClone {
    fn clone_box(&self) ->StatementNode;
}
impl<T: 'static + Statement + Clone> StatementClone for T {
    fn clone_box(&self) -> StatementNode {
        Box::new(self.clone())
    }
}
impl Clone for StatementNode {
    fn clone(&self) -> StatementNode {
        self.clone_box()
    }
}

#[derive(Clone, Debug)]
pub struct PrintNode {
    pub items: Vec<ExpressionNode>,
    pub newline: bool
}

#[derive(Clone, Debug)]
pub struct LetUnletNode {
    pub line: usize,
    pub col: usize,
    pub is_unlet: bool,
    pub name: String,
    pub rhs: ExpressionNode
}

#[derive(Clone, Debug)]
pub struct RefUnrefNode {
    pub line: usize,
    pub col: usize,
    pub is_unref: bool,
    pub name: String,
    pub rhs: LookupNode
}

#[derive(Clone, Debug)]
pub struct ModopNode {
    pub lookup: LookupNode,
    pub op: interpreter::Instruction,
    pub rhs: ExpressionNode
}

#[derive(Clone, Debug)]
pub struct PushPullNode {
    pub line: usize,
    pub col: usize,
    pub is_push: bool,
    pub name: String,
    pub lookup: LookupNode
}

#[derive(Clone, Debug)]
pub struct IfNode {
    pub fwd_expr: ExpressionNode,
    pub if_stmts: Vec<StatementNode>,
    pub else_stmts: Vec<StatementNode>,
    pub bkwd_expr: ExpressionNode
}

#[derive(Clone, Debug)]
pub struct WhileNode {
    pub fwd_expr: ExpressionNode,
    pub stmts: Vec<StatementNode>,
    pub bkwd_expr: Option<ExpressionNode>
}

#[derive(Clone, Debug)]
pub struct ForNode {
    pub iter_var: String,
    pub iterator: LookupNode,
    pub stmts: Vec<StatementNode>
}

#[derive(Clone, Debug)]
pub struct DoYieldNode {
    pub do_stmts: Vec<StatementNode>,
    pub yield_stmts: Vec<StatementNode>
}


#[derive(Clone, Debug)]
pub struct CatchNode {
    pub expr: ExpressionNode
}

#[derive(Clone, Debug)]
pub struct CallNode {
    pub is_uncall: bool,
    pub line: usize,
    pub col: usize,
    pub name: String,
    pub borrow_args: Vec<LookupNode>,
    pub stolen_args: Vec<String>,
    pub return_args: Vec<String>
}

#[derive(Clone, Debug)]
pub struct FunctionParam {
    pub name: String,
    pub is_ref: bool,
    pub link: Option<String>
}

#[derive(Clone, Debug)]
pub struct FunctionNode {
    pub name: String,
    pub owned_links: Vec<String>,
    pub borrow_params: Vec<FunctionParam>,
    pub steal_params: Vec<FunctionParam>,
    pub return_params: Vec<FunctionParam>,
    pub stmts: Vec<StatementNode>
}

#[derive(Clone, Debug)]
pub struct Module {
    pub global_func: FunctionNode,
    pub functions: Vec<FunctionNode>
}