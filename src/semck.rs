use error::CheckErrorKind;
use parser::Node;
use parser::Place;
use std::mem;

type Check = Result<(), CheckErrorKind>;

#[derive(Debug, Clone)]
pub struct SemChecker {
  in_loop: bool,
}

impl SemChecker {
  pub fn new() -> SemChecker {
    SemChecker { in_loop: false }
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

      Node::Block(ref mut ls) => {
        self.check_ifs(ls)?;

        for mut n in ls {
          self.check(&mut n)?;
        }
      }

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

      Node::ElseIf {
        ref mut cond,
        ref mut body,
      } => {
        self.check(cond)?;
        self.check(body)?;
      }

      Node::Else { ref mut body } => {
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

    Ok(())
  }

  fn insert_else(&self, node: &mut Node, new_else: Node) {
    if let Node::If {
      cond: _,
      body: _,
      ref mut els,
    } = node
    {
      if els.is_none() {
        *els = Some(Box::new(new_else));
      } else {
        self.insert_else(els.as_mut().unwrap(), new_else);
      }
    }
  }

  fn check_ifs(&self, block: &mut Vec<Node>) -> Check {
    let mut old_block = Vec::new();
    mem::swap(&mut old_block, block);

    let mut has_if = false;

    for n in old_block {
      match n {
        Node::If {
          cond: _,
          body: _,
          els: _,
        } => {
          has_if = true;
          block.push(n);
        }

        Node::ElseIf { ref cond, ref body } => {
          if !has_if {
            return Err(CheckErrorKind::MissingIf);
          }

          let mut popped = block.pop();
          match popped {
            Some(Node::If {
              cond: _,
              body: _,
              els: _,
            }) => {
              let mut unwrapped = popped.unwrap();

              self.insert_else(
                &mut unwrapped,
                Node::If {
                  cond: cond.clone(),
                  body: body.clone(),
                  els: None,
                },
              );

              block.push(unwrapped);
            }
            _ => return Err(CheckErrorKind::MissingIf),
          }
        }

        Node::Else { ref body } => {
          if !has_if {
            return Err(CheckErrorKind::MissingIf);
          }

          has_if = false;

          let mut popped = block.pop();
          match popped {
            Some(Node::If {
              cond: _,
              body: _,
              els: _,
            }) => {
              let mut unwrapped = popped.unwrap();

              self.insert_else(&mut unwrapped, (**body).clone());

              block.push(unwrapped);
            }
            _ => return Err(CheckErrorKind::MissingIf),
          }
        }
        x => block.push(x),
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
