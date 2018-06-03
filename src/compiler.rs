use code::Instr;
use data::Const;
use error::CompileErrorKind;
use lexer::Token;
use parser::Node;
use parser::Place;
use parser::Var;

type Compile = Result<(), CompileErrorKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiler {
  pub block: Vec<Instr>,
  pub consts: Vec<Const>,
  pub funcs: Vec<Instr>,
}

impl Compiler {
  pub fn new() -> Compiler {
    Compiler {
      block: Vec::new(),
      consts: Vec::new(),
      funcs: Vec::new(),
    }
  }

  pub fn get_const(&mut self, data: Const) -> usize {
    for (i, k) in self.consts.iter().enumerate() {
      if *k == data {
        return i;
      }
    }

    self.consts.push(data);
    return self.consts.len() - 1;
  }

  pub fn compile(&mut self, root: &Node) -> Compile {
    let const_id = self.get_const(Const::Null);
    self.block = self.compile_block(root)?;
    self.block.push(Instr::PushConst(const_id));
    Ok(())
  }

  fn compile_block(&mut self, root: &Node) -> Result<Vec<Instr>, CompileErrorKind> {
    let mut block = Vec::new();
    self.compile_aux(root, &mut block)?;
    Ok(block)
  }

  fn compile_aux(&mut self, root: &Node, block: &mut Vec<Instr>) -> Compile {
    match *root {
      Node::Null => {
        let const_id = self.get_const(Const::Null);
        block.push(Instr::PushConst(const_id));
      }

      Node::Int(x) => {
        let const_id = self.get_const(Const::Int(x));
        block.push(Instr::PushConst(const_id));
      }

      Node::Float(x) => {
        let const_id = self.get_const(Const::Float(x));
        block.push(Instr::PushConst(const_id));
      }

      Node::Bool(x) => {
        let const_id = self.get_const(Const::Bool(x));
        block.push(Instr::PushConst(const_id));
      }

      Node::Str(ref x) => {
        let const_id = self.get_const(Const::Str(x.clone()));
        block.push(Instr::PushConst(const_id));
      }

      Node::Name(ref x) => {
        let const_id = self.get_const(Const::Str(x.clone()));
        block.push(Instr::PushConst(const_id));
        block.push(Instr::PushScope);
        block.push(Instr::Get);
      }

      Node::Super(count, ref x) => {
        self.compile_aux(x, block)?;
        block.push(Instr::PushScope);
        for _ in 0..count {
          block.push(Instr::UnOp(Token::Mul));
        }
        block.push(Instr::Get);
      }

      Node::Index { ref lhs, ref rhs } => {
        self.compile_aux(rhs, block)?;
        self.compile_aux(lhs, block)?;
        block.push(Instr::Get);
      }

      Node::Local => {
        block.push(Instr::PushScope);
      }

      Node::Expr(ref bx) => {
        self.compile_aux(bx, block)?;
        block.push(Instr::Pop);
      }

      Node::Block(ref ls) => for n in ls {
        self.compile_aux(n, block)?;
      },

      Node::Catch { ref body } => {
        let mut new_block = self.compile_block(body)?;
        let const_id = self.get_const(Const::Null);
        new_block.push(Instr::PushConst(const_id));
        block.push(Instr::Catch(new_block));
      }

      Node::Assn { ref lhs, ref rhs } => {
        if let Place::Single(ref place) = *lhs {
          self.compile_aux(rhs, block)?;
          self.compile_place_single(place, block)?;
          block.push(Instr::Set);
        } else {
          panic!("can't use multi places");
        }
      }

      Node::LogicExpr { ref nodes, ref ops } => {
        let mut new_block = Vec::new();
        self.compile_aux(&nodes[0], &mut new_block)?;
        for (op, node) in ops.iter().zip(&nodes[1..]) {
          new_block.push(Instr::LogicOp(op.clone()));
          self.compile_aux(&node, &mut new_block)?;
        }
        block.push(Instr::Returnable(new_block));
      }

      Node::CmpExpr { ref nodes, ref ops } => {
        if nodes.len() == 2 {
          self.compile_aux(&nodes[0], block)?;
          self.compile_aux(&nodes[1], block)?;
          block.push(Instr::CmpOp(ops[0].clone(), false));
        } else {
          let mut new_block = Vec::new();
          self.compile_aux(&nodes[0], &mut new_block)?;
          let end = ops.len() - 1;
          for (i, (op, node)) in ops.iter().zip(&nodes[1..]).enumerate() {
            self.compile_aux(&node, &mut new_block)?;
            new_block.push(Instr::CmpOp(op.clone(), i != end));
          }
          block.push(Instr::Returnable(new_block));
        }
      }

      Node::BinExpr {
        ref lhs,
        ref op,
        ref rhs,
      } => {
        self.compile_aux(lhs, block)?;
        self.compile_aux(rhs, block)?;
        block.push(Instr::BinOp(op.clone()));
      }

      Node::UnExpr { ref op, ref val } => {
        self.compile_aux(val, block)?;
        block.push(Instr::UnOp(op.clone()));
      }

      Node::Break => {
        block.push(Instr::Break);
      }

      Node::Continue => {
        block.push(Instr::Continue);
      }

      Node::Pass => {}

      Node::Loop { ref body } => {
        let body_block = self.compile_block(body)?;
        block.push(Instr::Loop(body_block));
      }

      Node::While { ref expr, ref body } => {
        let mut expr_block = Vec::new();
        self.compile_aux(expr, &mut expr_block)?;
        let body_block = self.compile_block(body)?;
        expr_block.push(Instr::IfElse(body_block, vec![Instr::Break]));
        block.push(Instr::Loop(expr_block));
      }

      Node::For {
        ref decl,
        ref expr,
        ref body,
      } => {
        self.compile_aux(expr, block)?;
        let mut iter_block = Vec::new();
        iter_block.push(Instr::Dup);
        iter_block.push(Instr::Call);
        iter_block.push(Instr::ForBreak);
        if let Var::Single(ref decl) = *decl {
          let const_id = self.get_const(Const::Str(decl.clone()));
          iter_block.push(Instr::PushConst(const_id));
          iter_block.push(Instr::PushScope);
          iter_block.push(Instr::Set);
        } else {
          panic!("can't use multi decls");
        }

        self.compile_aux(body, &mut iter_block)?;
        block.push(Instr::Loop(iter_block));
        block.push(Instr::Pop);
      }

      Node::If {
        ref cond,
        ref body,
        ref els,
      } => {
        let if_block = self.compile_block(body)?;
        self.compile_aux(cond, block)?;
        match *els {
          Some(ref els) => {
            let els_block = self.compile_block(els)?;
            block.push(Instr::IfElse(if_block, els_block));
          }
          _ => {
            block.push(Instr::If(if_block));
          }
        }
      }

      Node::Else { body: _ } => {
        // this is a nop; semck transformations remove these nodes
      }

      Node::FuncDef {
        ref params,
        ref body,
      } => {
        // value -> key -> table
        let mut new_block = Vec::new();
        for par in params {
          let const_id = self.get_const(Const::Str(par.clone()));
          new_block.push(Instr::PushConst(const_id));
          new_block.push(Instr::PushScope);
          new_block.push(Instr::Set);
        }
        self.compile_aux(body, &mut new_block)?;
        let const_id = self.get_const(Const::Null);
        new_block.push(Instr::PushConst(const_id));
        self.funcs.push(Instr::Returnable(new_block));
        block.push(Instr::PushFunc(self.funcs.len() - 1));
      }

      Node::Return(ref val) => {
        match *val {
          None => {
            let const_id = self.get_const(Const::Null);
            block.push(Instr::PushConst(const_id));
          }
          Some(ref x) => {
            self.compile_aux(x, block)?;
          }
        }
        block.push(Instr::Return);
      }

      Node::FuncCall { ref func, ref args } => {
        for arg in args.iter().rev() {
          self.compile_aux(arg, block)?;
        }
        self.compile_aux(func, block)?;
        block.push(Instr::Call);
      }

      Node::MethodCall {
        ref owner,
        ref method,
        ref args,
      } => {
        for arg in args.iter().rev() {
          self.compile_aux(arg, block)?;
        }
        self.compile_aux(method, block)?;
        self.compile_aux(owner, block)?;
        block.push(Instr::MethodGet);
        block.push(Instr::Call);
      }

      _ => {
        println!("WARNING: unable to compile node: {:?}", root);
      }
    }

    Ok(())
  }

  fn compile_place_single(&mut self, place: &Node, block: &mut Vec<Instr>) -> Compile {
    // stable Rust doesn't let you destructure boxes in patterns
    // but it does let you destructure references, which boxes can
    // be coerced into...

    match *place {
      Node::Name(ref name) => {
        let const_id = self.get_const(Const::Str(name.clone()));
        block.push(Instr::PushConst(const_id));
        block.push(Instr::PushScope);
      }
      Node::Index { ref lhs, ref rhs } => {
        self.compile_aux(rhs, block)?;
        self.compile_aux(lhs, block)?;
      }
      Node::Super(count, ref rhs) => {
        self.compile_aux(rhs, block)?;
        block.push(Instr::PushScope);
        for _ in 0..count {
          block.push(Instr::UnOp(Token::Mul));
        }
      }
      _ => {}
    }
    Ok(())
  }
}
