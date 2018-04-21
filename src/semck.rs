use parser::Node;

type Check = Result<(), CheckErrorKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum CheckErrorKind {
  NotInLoop,
  MissingIf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemChecker {
  in_loop: bool,
  has_if: bool,
}

impl SemChecker {
  pub fn new() -> SemChecker {
    SemChecker {
      in_loop: false,
      has_if: false,
    }
  }

  pub fn check(&mut self, node: &mut Node) -> Check {
    println!("checking: {:?}", node);
    match *node {
      Node::Stmt(ref mut bx) => {
        self.check(bx)?;
      }

      Node::Block(ref mut ls) | Node::Catch(ref mut ls) => for mut n in ls {
        self.check(&mut n)?;
      },

      Node::Loop { ref mut body } => {
        self.in_loop = true;
        for mut n in body {
          self.check(&mut n)?;
        }
        self.in_loop = false;
      }

      Node::While {
        ref mut body,
        ref expr,
      } => {
        self.in_loop = true;
        for mut n in body {
          self.check(&mut n)?;
        }
        self.in_loop = false;
      }

      Node::For {
        ref mut body,
        ref decl,
        ref expr,
      } => {
        self.in_loop = true;
        for mut n in body {
          self.check(&mut n)?;
        }
        self.in_loop = false;
      }

      Node::Break | Node::Continue => {
        if !self.in_loop {
          return Err(CheckErrorKind::NotInLoop);
        }
      }

      // TODO add if-else if-else checks
      // TODO add assn checks
      _ => {}
    }

    Ok(())
  }
}
