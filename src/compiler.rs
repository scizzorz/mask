use code::Data;
use code::Instr;
use parser::Node;
use parser::Place;

type Compile = Result<(), CompileErrorKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileErrorKind {
  MissingCurrentBlock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
  pub instrs: Vec<Instr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compiler {
  pub block: Block,
  pub consts: Vec<Data>,
}

impl Block {
  pub fn new() -> Block {
    Block { instrs: Vec::new() }
  }

  pub fn add(&mut self, instr: Instr) {
    self.instrs.push(instr);
  }
}

impl Compiler {
  pub fn new() -> Compiler {
    Compiler {
      block: Block::new(),
      consts: Vec::new(),
    }
  }

  pub fn get_const(&mut self, data: Data) -> usize {
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

  fn compile_block(&mut self, root: &Node) -> Result<Block, CompileErrorKind> {
    let mut block = Block::new();
    self.compile_aux(root, &mut block)?;
    Ok(block)
  }

  fn compile_aux(&mut self, root: &Node, block: &mut Block) -> Compile {
    match *root {
      Node::Null => {
        let const_id = self.get_const(Data::Null);
        block.add(Instr::PushConst(const_id));
      }

      Node::Int(x) => {
        let const_id = self.get_const(Data::Int(x));
        block.add(Instr::PushConst(const_id));
      }

      Node::Float(x) => {
        let const_id = self.get_const(Data::Float(x));
        block.add(Instr::PushConst(const_id));
      }

      Node::Bool(x) => {
        let const_id = self.get_const(Data::Bool(x));
        block.add(Instr::PushConst(const_id));
      }

      Node::Str(ref x) => {
        let const_id = self.get_const(Data::Str(x.clone()));
        block.add(Instr::PushConst(const_id));
      }

      Node::Expr(ref bx) => {
        self.compile_aux(bx, block)?;
        block.add(Instr::Pop);
      }

      Node::Block(ref ls) => for n in ls {
        self.compile_aux(n, block)?;
      },

      Node::Catch { ref body } => {
        let new_block = self.compile_block(body)?;
        block.add(Instr::Block(new_block.instrs));
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
            block.add(Instr::IfElse(if_block.instrs, els_block.instrs));
          }
          _ => {
            block.add(Instr::If(if_block.instrs));
          }
        }
      }

      _ => {}
    }

    Ok(())
  }
}
