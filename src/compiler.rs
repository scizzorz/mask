use code::Data;
use code::Instr;
use parser::Node;
use parser::Place;

type Compile = Result<(), CompileErrorKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileErrorKind {
  MissingCurrentBlock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  instrs: Vec<Instr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compiler {
  blocks: Vec<Block>,
}

impl Block {
  pub fn new() -> Block {
    Block {
      instrs: Vec::new(),
    }
  }

  pub fn add(&mut self, instr: Instr) {
    self.instrs.push(instr);
  }
}

impl Compiler {
  pub fn new() -> Compiler {
    Compiler {
      blocks: Vec::new(),
    }
  }

  pub fn compile(&mut self, root: &mut Node) -> Compile {
    Ok(())
  }

  fn new_block(&mut self) {
    self.blocks.push(Block::new());
  }

  fn add_block(&mut self, block: Block) {
    self.blocks.push(block);
  }
}
