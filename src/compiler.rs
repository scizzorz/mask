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
  consts: Vec<Data>,
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
      blocks: Vec::new(),
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
    let mut block = Block::new();
    println!("compiling: {:?}", root);
    match *root {
      Node::Null => {
        let const_id = self.get_const(Data::Null);
        println!("const_id: {}", const_id);
      }

      Node::Int(x) => {
        let const_id = self.get_const(Data::Int(x));
        println!("const_id: {}", const_id);
      }

      Node::Float(x) => {
        let const_id = self.get_const(Data::Float(x));
        println!("const_id: {}", const_id);
      }

      Node::Bool(x) => {
        let const_id = self.get_const(Data::Bool(x));
        println!("const_id: {}", const_id);
      }

      Node::Str(ref x) => {
        let const_id = self.get_const(Data::Str(x.clone()));
        println!("const_id: {}", const_id);
      }

      Node::Expr(ref bx) => {
        self.compile(bx)?;
      }

      Node::Block(ref ls) => for n in ls {
        self.compile(n)?;
      },
      _ => {}
    }
    self.add_block(block);
    Ok(())
  }

  fn new_block(&mut self) {
    self.blocks.push(Block::new());
  }

  fn add_block(&mut self, block: Block) {
    self.blocks.push(block);
  }

  pub fn get_instrs(&self) -> Vec<Instr> {
    let mut instrs = Vec::new();
    for block in &self.blocks {
      for instr in &block.instrs {
        instrs.push(instr.clone());
      }
    }
    instrs
  }
}
