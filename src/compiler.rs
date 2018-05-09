use data::Const;
use data::Data;
use code::Instr;
use parser::Node;
use parser::Place;

type Compile = Result<(), CompileErrorKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileErrorKind {
  MissingCurrentBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiler {
  pub block: Vec<Instr>,
  pub consts: Vec<Const>,
}

impl Compiler {
  pub fn new() -> Compiler {
    Compiler {
      block: Vec::new(),
      consts: Vec::new(),
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
    self.block = self.compile_block(root)?;
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

      Node::Index { ref lhs, ref rhs } => {
        self.compile_aux(rhs, block)?;
        self.compile_aux(lhs, block)?;
        block.push(Instr::Get);
      }

      Node::Table => {
        block.push(Instr::NewTable);
      }

      Node::Expr(ref bx) => {
        self.compile_aux(bx, block)?;
        block.push(Instr::Pop);
      }

      Node::Block(ref ls) => for n in ls {
        self.compile_aux(n, block)?;
      },

      Node::Catch { ref body } => {
        let new_block = self.compile_block(body)?;
        block.push(Instr::Block(new_block));
      }

      Node::Print { ref expr } => {
        self.compile_aux(expr, block)?;
        block.push(Instr::Print);
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
        block.push(Instr::BinOp(op.clone()));
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
      _ => {}
    }
    Ok(())
  }
}
