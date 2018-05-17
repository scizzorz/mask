use parser::Node;
use parser::Place;
use lexer::Token;

type Check = Result<(), CheckErrorKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckErrorKind {
  NotInLoop,
  MissingIf,
  NotPlace,
}

#[derive(Debug, Clone)]
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
    match *node {
      Node::Expr(ref mut bx) => {
        self.check(bx)?;
      }

      Node::FuncDef {
        ref mut body,
        params: _,
      } => {
        self.check(body)?;
      }

      Node::Block(ref mut ls) => for mut n in ls {
        self.check(&mut n)?;
      },

      Node::Catch { ref mut body } => {
        self.check(body)?;
      }

      Node::Loop { ref mut body } => {
        self.in_loop = true;
        self.check(body)?;
        self.in_loop = false;
      }

      Node::While {
        ref mut body,
        ref mut expr,
      } => {
        self.check(expr)?;
        self.in_loop = true;
        self.check(body)?;
        self.in_loop = false;
      }

      Node::For {
        ref mut body,
        decl: _,
        expr: _,
      } => {
        self.in_loop = true;
        self.check(body)?;
        self.in_loop = false;
      }

      Node::If {
        ref mut cond,
        ref mut body,
        els: _,
      } => {
        self.check(cond)?;
        self.check(body)?;
      }

      Node::Break | Node::Continue => {
        if !self.in_loop {
          return Err(CheckErrorKind::NotInLoop);
        }
      }

      Node::Assn {
        ref lhs,
        ref mut rhs,
      } => {
        self.check_place(lhs)?;
        self.check(rhs)?;
      }

      // TODO add if-elif-else checks
      _ => {}
    }

    match *node {
      Node::If {
        cond: _,
        body: _,
        els: _,
      } => {
        self.has_if = true;
      }
      Node::ElseIf {
        cond: _,
        body: _,
      } => {
        if !self.has_if {
          return Err(CheckErrorKind::MissingIf);
        }
      }
      Node::Else {
        body: _,
      } => {
        if !self.has_if {
          return Err(CheckErrorKind::MissingIf);
        }
      }
      _ => {
        self.has_if = false;
      }
    }


    Ok(())
  }

  fn check_place(&self, place: &Place) -> Check {
    match *place {
      Place::Single(ref node) => {
        self.is_place(node)?;
      }
      Place::Multi(ref places) => for pl in places {
        self.check_place(&pl)?;
      },
    };

    Ok(())
  }

  fn is_place(&self, node: &Node) -> Check {
    match *node {
      Node::Name(_) | Node::Index { lhs: _, rhs: _ } | Node::Super(_, _) => Ok(()),
      _ => Err(CheckErrorKind::NotPlace),
    }
  }
}
