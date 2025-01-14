use std::collections::HashSet;

use crate::syntaxtree as ST;
use crate::syntaxtree::Expression as STExpression;
use crate::interpreter;
use interpreter::Instruction;


#[derive(Clone, Default, Debug)]
pub struct Code {
    fwd: Vec<Instruction>,
    bkwd: Vec<Instruction>,
    f2b_links: Vec<(usize, usize)>,
    b2f_links: Vec<(usize, usize)>
}

impl Code {
    pub fn new() -> Code {
        Default::default()
    }

    pub fn with_capacity(l1: usize, l2: usize) -> Code {
        Code{
            fwd: Vec::with_capacity(l1),
            bkwd: Vec::with_capacity(l2),
            f2b_links: Vec::new(),
            b2f_links: Vec::new()
        }
    }

    pub fn link_fwd2bkwd(&mut self) {
        self.f2b_links.push((self.fwd.len(), self.bkwd.len()));
        // Insert dummy instruction //
        self.fwd.push(Instruction::Reverse{idx: 0});
    }
    
    pub fn link_bkwd2fwd(&mut self) {
        self.b2f_links.push((self.bkwd.len(), self.fwd.len()));
        // Insert dummy instruction //
        self.bkwd.push(Instruction::Reverse{idx: 0});
    }

    pub fn push_fwd(&mut self, x: Instruction) {
        self.fwd.push(x);
    }

    pub fn push_bkwd(&mut self, x: Instruction) {
        self.bkwd.push(x);
    }

    pub fn append_fwd(&mut self, mut instructions: Vec<Instruction>) {
        self.fwd.append(&mut instructions);
    }
    
    pub fn append_bkwd(&mut self, instructions: Vec<Instruction>) {
        self.bkwd.extend(instructions.into_iter().rev());
    }

    pub fn fwd_len(&mut self) -> usize {
        self.fwd.len()
    }

    pub fn bkwd_len(&mut self) -> usize {
        self.bkwd.len()
    }

    pub fn clear_bkwd(&mut self) {
        if self.bkwd.len() == 0 {return};
        for instruction in self.bkwd.drain(..) {
            if let Instruction::Reverse{idx: _} = instruction {
                panic!("Internal inconsistency: clear_bkwd called on a Reverse instruction");
            }
        }
    }

    pub fn extend(&mut self, other: Code) {
        let Code{fwd, bkwd, f2b_links, b2f_links} = other;
        let (flen, blen) = (self.fwd.len(), self.bkwd.len());
        self.fwd.extend(fwd);
        self.bkwd.extend(bkwd);
        for (f, b) in f2b_links.into_iter() {
            self.f2b_links.push((f + flen, b + blen));
        }
        for (b, f) in b2f_links.into_iter() {
            self.b2f_links.push((b + blen, f + flen));
        }
    }

    pub fn reversed(mut self) -> Code {
        for (f, b) in self.f2b_links.iter_mut() {
            *f = self.fwd.len() - *f;
            *b = self.bkwd.len() - *b;
        }
        for (b, f) in self.b2f_links.iter_mut() {
            *f = self.fwd.len() - *f;
            *b = self.bkwd.len() - *b;
        }
        self.bkwd.reverse();
        self.fwd.reverse();
        Code{
            fwd: self.bkwd,
            bkwd: self.fwd,
            f2b_links: self.b2f_links,
            b2f_links: self.f2b_links
        }
    }

    pub fn finalise(code: Code) -> interpreter::Code {
        let Code{mut fwd, mut bkwd, f2b_links, b2f_links} = code;
        bkwd.reverse();

        // Compute instruction pointers for reversals //
        for (f, b) in f2b_links.into_iter() {
            let b = bkwd.len() - b;
            match fwd[f] {
                Instruction::Reverse{idx: _} => fwd[f] = Instruction::Reverse{idx: b},
                _ => panic!()
            }
        }
        for (b, f) in b2f_links.into_iter() {
            let b = bkwd.len() - b;
            match bkwd[b] {
                Instruction::Reverse{idx: _} => bkwd[b] = Instruction::Reverse{idx: f},
                _ => panic!()
            }
        }

        // Replace relative jumps with absolute jumps //
        for i in 0..fwd.len() {
            match fwd[i] {
                Instruction::RelativeJump{delta} => {
                    fwd[i] = Instruction::Jump{ip: (i as isize + delta) as usize}
                },
                Instruction::RelativeJumpIfTrue{delta} => {
                    fwd[i] = Instruction::JumpIfTrue{ip: (i as isize + delta) as usize}
                },
                Instruction::RelativeJumpIfFalse{delta} => {
                    fwd[i] = Instruction::JumpIfFalse{ip: (i as isize + delta) as usize}
                },
                Instruction::StepIter{ip} => {
                    fwd[i] = Instruction::StepIter{ip: i + ip}
                },
                _ => {}
            }
        }
        for i in 0..bkwd.len() {
            match bkwd[i] {
                Instruction::RelativeJump{delta} => {
                    bkwd[i] = Instruction::Jump{ip: (i as isize + delta) as usize}
                },
                Instruction::RelativeJumpIfTrue{delta} => {
                    bkwd[i] = Instruction::JumpIfTrue{ip: (i as isize + delta) as usize}
                },
                Instruction::RelativeJumpIfFalse{delta} => {
                    bkwd[i] = Instruction::JumpIfFalse{ip: (i as isize + delta) as usize}
                },
                Instruction::StepIter{ip} => {
                    bkwd[i] = Instruction::StepIter{ip: i + ip}
                },
                _ => {}
            }
        }
        interpreter::Code{fwd, bkwd}
    }
}


impl ST::Expression for ST::FractionNode {
    fn is_mono(&self) -> bool {false}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars}

    fn compile(&self) -> Vec<Instruction> {
        vec![Instruction::LoadConst{idx: self.const_idx}]
    }
}

impl ST::Expression for ST::StringNode {
    fn is_mono(&self) -> bool {false}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars}

    fn compile(&self) -> Vec<Instruction> {
        vec![Instruction::LoadConst{idx: self.const_idx}]
    }
}

impl ST::Expression for ST::LookupNode {
    fn is_mono(&self) -> bool {self.is_mono}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars}

    fn compile(&self) -> Vec<Instruction> {
        let mut instructions = Vec::with_capacity(self.indices.len()+1);        
        for index in self.indices.iter().rev() {
            instructions.extend(index.compile());
        }

        if self.is_global {
            instructions.push(Instruction::LoadGlobalRegister{register:self.register});
        } else {
            instructions.push(Instruction::LoadRegister{register:self.register});
        }

        if !self.indices.is_empty() {
            instructions.push(Instruction::Subscript{size: self.indices.len()});
        }
        instructions
    }
}

impl ST::Expression for ST::BinopNode {
    fn is_mono(&self) -> bool {self.is_mono}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars}

    fn compile(&self) -> Vec<Instruction> {
        let mut ret = Vec::new();
        let lhs = self.lhs.compile();
        let rhs = self.rhs.compile();
        if self.op == Instruction::BinopAnd {
            ret.extend(lhs);
            ret.push(Instruction::RelativeJumpIfTrue{delta: 3});
            ret.push(Instruction::CreateInt{val: 0}); // Set False
            ret.push(Instruction::RelativeJump{delta: (rhs.len() + 1) as isize});
            ret.extend(rhs);
        } else if self.op == Instruction::BinopOr {
            ret.extend(lhs);
            ret.push(Instruction::RelativeJumpIfFalse{delta: 3});
            ret.push(Instruction::CreateInt{val: 1}); // Set True
            ret.push(Instruction::RelativeJump{delta: (rhs.len() + 1) as isize});
            ret.extend(rhs);
        } else {
            ret.extend(lhs);
            ret.extend(rhs);
            ret.push(self.op.clone());
        }
        ret
    }
}

impl ST::Expression for ST::UniopNode {
    fn is_mono(&self) -> bool {self.is_mono}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars} // TODO: can I provide a type-generic implementation?

    fn compile(&self) -> Vec<Instruction> {
        let mut ret = Vec::new();
        ret.extend(self.expr.compile());
        ret.push(self.op.clone());
        ret
    }
}

impl ST::Expression for ST::ArrayLiteralNode {
    fn is_mono(&self) -> bool {self.is_mono}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars}
    
    fn compile(&self) -> Vec<Instruction> {
        let mut ret = Vec::with_capacity(self.items.len() + 1);
        for item in self.items.iter().rev() {
            ret.extend(item.compile());
        }
        ret.push(Instruction::ArrayLiteral{size: self.items.len()});
        ret
    }
}

impl ST::Expression for ST::ArrayRepeatNode {
    fn is_mono(&self) -> bool {self.is_mono}
    fn used_vars(&self) -> &HashSet<isize> {&self.used_vars}
    
    fn compile(&self) -> Vec<Instruction> {
        let mut ret = self.item.compile();
        ret.extend(self.dimensions.compile());
        ret.push(Instruction::ArrayRepeat);
        ret
    }
}


// ------------------------------ Statement Nodes ------------------------------ //

impl ST::Statement for ST::PrintNode {
    fn is_mono(&self) -> bool {true}

    fn compile(&self) -> Code {
        let mut count = self.items.len() as isize;
        if self.newline {count *= -1};

        let mut code = Code::new();

        for item in self.items.iter().rev() {
            code.append_fwd(item.compile());
        }
        code.push_fwd(Instruction::Print{count});

        code.push_bkwd(Instruction::Print{count});
        for item in self.items.iter() {
            code.append_bkwd(item.compile());
        }
        
        if self.is_mono {code.clear_bkwd();}
        code
    }
}


impl ST::Statement for ST::LetUnletNode {
    fn is_mono(&self) -> bool {self.is_mono}

    fn compile(&self) -> Code {
        let mut code = Code::new();
        if self.is_unlet {
            code.push_fwd(Instruction::FreeRegister{register: self.register});

            code.push_bkwd(Instruction::StoreRegister{register: self.register});
            code.push_bkwd(Instruction::UniqueVar);
            code.append_bkwd(self.rhs.compile());
        } else {
            code.append_fwd(self.rhs.compile());
            code.push_fwd(Instruction::UniqueVar);
            code.push_fwd(Instruction::StoreRegister{register: self.register});

            code.push_bkwd(Instruction::FreeRegister{register: self.register});
        }

        if self.is_mono {code.clear_bkwd();}
        code
    }
}


impl ST::Statement for ST::RefUnrefNode {
    fn is_mono(&self) -> bool {self.is_mono}

    fn compile(&self) -> Code {
        let mut create_ref = self.rhs.compile();
        create_ref.push(Instruction::StoreRegister{register: self.register});
        let remove_ref = vec![Instruction::FreeRegister{register: self.register}];

        let mut code = Code::new();
        if self.is_unref{
            code.append_fwd(remove_ref);
            code.append_bkwd(create_ref);
        } else {
            code.append_fwd(create_ref);
            code.append_bkwd(remove_ref);
        }

        if self.is_mono {code.clear_bkwd();}
        code
    }
}


impl ST::Statement for ST::ModopNode {
    fn is_mono(&self) -> bool {self.is_mono}

    fn compile(&self) -> Code {
        let lookup = self.lookup.compile();
        let rhs = self.rhs.compile();
        let bkwd_op = match self.op {
            Instruction::BinopAdd => Instruction::BinopSub,
            Instruction::BinopSub => Instruction::BinopAdd,
            Instruction::BinopMul => Instruction::BinopDiv,
            Instruction::BinopDiv => Instruction::BinopMul,
            _ => unreachable!()
        };

        let capacity = lookup.len() + rhs.len() + 3;
        let mut code = Code::with_capacity(capacity, capacity);

        code.append_fwd(lookup.clone());
        code.push_fwd(Instruction::DuplicateRef);
        code.append_fwd(rhs.clone());
        code.push_fwd(self.op.clone());
        code.push_fwd(Instruction::Store);

        code.push_bkwd(Instruction::Store);
        code.push_bkwd(bkwd_op);
        code.append_bkwd(rhs);
        code.push_bkwd(Instruction::DuplicateRef);
        code.append_bkwd(lookup);
        
        if self.is_mono {code.clear_bkwd();}
        code
    }
}

impl ST::Statement for ST::PushPullNode {
    fn is_mono(&self) -> bool {self.is_mono}
    
    fn compile(&self) -> Code {
        let mut code = Code::new();
        let lookup = self.lookup.compile();
        let register = self.register;

        if self.is_push {
            code.append_fwd(lookup.clone());
            code.push_fwd(Instruction::Push{register});
    
            code.push_bkwd(Instruction::Pull{register});
            code.append_bkwd(lookup);

        } else {
            code.append_fwd(lookup.clone());
            code.push_fwd(Instruction::Pull{register});
    
            code.push_bkwd(Instruction::Push{register});
            code.append_bkwd(lookup);
        }
        
        if self.is_mono {code.clear_bkwd();}
        code
    }
}


impl ST::Statement for ST::IfNode {
    fn is_mono(&self) -> bool {self.is_mono}
    
    fn compile(&self) -> Code {
        let fwd_expr = self.fwd_expr.compile();
        let bkwd_expr = self.bkwd_expr.compile();
        let mut if_block = Code::new();
        for stmt in self.if_stmts.iter() {
            if_block.extend(stmt.compile());
        }
        let mut else_block = Code::new();
        for stmt in self.else_stmts.iter() {
            else_block.extend(stmt.compile());
        }
        let if_bkwd_len = if_block.bkwd_len() as isize;
        let else_bkwd_len = else_block.bkwd_len() as isize;
        
        let mut code = Code::with_capacity(
            if_block.fwd_len() + else_block.fwd_len() + fwd_expr.len() + 2, 
            if_block.bkwd_len() + else_block.bkwd_len() + bkwd_expr.len() + 2
        );
        
        code.append_fwd(fwd_expr);
        code.push_fwd(Instruction::RelativeJumpIfFalse{
            delta: (if_block.fwd_len() + 2) as isize
        });
        code.extend(if_block);
        code.push_fwd(Instruction::RelativeJump{
            delta: (else_block.fwd_len() + 1) as isize
        });
        code.push_bkwd(Instruction::RelativeJump{delta: if_bkwd_len + 1});
        code.extend(else_block);
        code.push_bkwd(Instruction::RelativeJumpIfTrue{delta: else_bkwd_len + 2});
        code.append_bkwd(bkwd_expr);

        if self.is_mono {code.clear_bkwd();}
        code
    }
}


impl ST::Statement for ST::WhileNode {
    fn is_mono(&self) -> bool {self.is_mono}
    
    fn compile(&self) -> Code {
        let fwd_expr = self.fwd_expr.compile();
        // The backward condition can be None if the loop is mono
        let bkwd_expr = self.bkwd_expr.as_ref().map(|e| e.compile());
        let mut stmts = Code::new();
        for stmt in self.stmts.iter() {
            stmts.extend(stmt.compile());
        }

        let stmts_fwd_len = stmts.fwd_len() as isize;
        let stmts_bkwd_len = stmts.bkwd_len() as isize;
        let fwd_expr_len = fwd_expr.len() as isize;

        let mut code = Code::new();
        
        code.append_fwd(fwd_expr);
        
        code.push_fwd(Instruction::RelativeJumpIfFalse{
            delta: stmts_fwd_len + 2
        });
        if let Some(bkwd_expr) = &bkwd_expr {
            code.push_bkwd(Instruction::RelativeJump{
                delta: -stmts_bkwd_len - (bkwd_expr.len() as isize) - 1
           })
        }; 

        code.extend(stmts);

        code.push_fwd(Instruction::RelativeJump{
            delta: -stmts_fwd_len - fwd_expr_len - 1
        });

        
        if let Some(bkwd_expr) = bkwd_expr {
           code.push_bkwd(Instruction::RelativeJumpIfFalse{
                delta: stmts_bkwd_len + 2
            });
            code.append_bkwd(bkwd_expr);
        };


        if self.is_mono {code.clear_bkwd();}
        code
    }
}


impl ST::Statement for ST::ForNode {
    fn is_mono(&self) -> bool {self.is_mono}
    
    fn compile(&self) -> Code {
        let iter_lookup = self.iterator.compile();

        let mut stmts = Code::new();
        for stmt in self.stmts.iter() {
            stmts.extend(stmt.compile());
        }
        let stmts_fwd_len = stmts.fwd_len();
        let stmts_bkwd_len = stmts.bkwd_len();

        let mut code = Code::new();
        
        code.append_fwd(iter_lookup.clone());
        code.push_fwd(Instruction::CreateIter{register: self.register});
        code.push_fwd(Instruction::StepIter{ip: stmts_fwd_len + 2});
        code.push_bkwd(Instruction::RelativeJump{delta: -(1 + stmts_bkwd_len as isize)});

        code.extend(stmts);

        code.push_fwd(Instruction::RelativeJump{delta: -(1 + stmts_fwd_len as isize)});
        code.push_bkwd(Instruction::StepIter{ip: stmts_bkwd_len + 2});
        code.push_bkwd(Instruction::CreateIter{register: self.register});
        code.append_bkwd(iter_lookup);
        
        if self.is_mono {code.clear_bkwd();}
        code
    }
}

impl ST::Statement for ST::DoYieldNode {
    fn is_mono(&self) -> bool {false}
    
    fn compile(&self) -> Code {

        let mut code = Code::new();
        for do_stmt in self.do_stmts.iter() {
            code.extend(do_stmt.compile());
        }
        let undo_block = code.clone().reversed();
        for yield_stmt in self.yield_stmts.iter() {
            code.extend(yield_stmt.compile());
        }
        code.extend(undo_block);
        
        code
    }
}


impl ST::Statement for ST::CatchNode {
    fn is_mono(&self) -> bool {true}
    
    fn compile(&self) -> Code {
        let mut code = Code::new();
        code.append_fwd(self.expr.compile());
        code.push_fwd(Instruction::RelativeJumpIfFalse{delta: 2});
        code.link_fwd2bkwd();
        code
    }
}

impl ST::Statement for ST::CallNode {
    fn is_mono(&self) -> bool {self.is_mono}
    
    fn compile(&self) -> Code {
        let mut code = Code::new();

        for &register in self.stolen_args.iter().rev() {
            code.push_fwd(Instruction::LoadRegister{register});
            code.push_fwd(Instruction::FreeRegister{register});
        }

        if self.is_uncall {
            code.push_bkwd(Instruction::Call{idx: self.func_idx});
            code.push_fwd(Instruction::Uncall{idx: self.func_idx});
        } else {
            for arg in self.borrow_args.iter().rev() {
                code.append_fwd(arg.compile());
            }
            code.push_fwd(Instruction::Call{idx: self.func_idx});
            code.push_bkwd(Instruction::Uncall{idx: self.func_idx});
        }

        for &register in self.return_args.iter().rev() {
            code.push_fwd(Instruction::StoreRegister{register});
        }

        if self.is_mono {code.clear_bkwd();}
        code
    }
}

impl ST::FunctionNode {
    pub fn compile(&self) -> interpreter::Function {
        let mut code = Code::new();

        for &register in &self.borrow_registers {
            code.push_fwd(Instruction::StoreRegister{register});
        }
        for &register in &self.steal_registers {
            code.push_fwd(Instruction::StoreRegister{register});
            code.push_bkwd(Instruction::LoadRegister{register});
        }

        for stmt in &self.stmts {
            code.extend(stmt.compile());
        }

        for &register in &self.return_registers {
            code.push_fwd(Instruction::LoadRegister{register});
            code.push_bkwd(Instruction::StoreRegister{register});
        }
        for &register in &self.borrow_registers {
            code.push_bkwd(Instruction::StoreRegister{register});
        }

        interpreter::Function{
            consts: self.consts.clone(),
            code: Code::finalise(code),
            num_registers: self.num_registers
        }
    }

    // Compile as the special 'global function' which is run for the global scope before main
    pub fn compile_to_global(&self) -> interpreter::Function {
        let mut func = self.compile();
        for instruction in func.code.fwd.iter_mut().chain(func.code.bkwd.iter_mut()) {
            match instruction {
                interpreter::Instruction::LoadRegister{register} => {
                    *instruction = interpreter::Instruction::LoadGlobalRegister{register: *register};
                },
                interpreter::Instruction::StoreRegister{register} => {
                    *instruction = interpreter::Instruction::StoreGlobalRegister{register: *register};
                }
                _ => {}
            }
        };
        func
    }
}

impl ST::Module {
    pub fn compile(&self) -> interpreter::Module {
        let main_idx = self.main_idx;
        let mut functions: Vec<_> = self.functions.iter().map(|f| f.compile()).collect();
        let global_func_idx = functions.len();
        functions.push(self.global_func.compile_to_global());

        interpreter::Module{main_idx, functions, global_func_idx}
    }
}