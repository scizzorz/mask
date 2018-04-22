use codemap::File;
use codemap::CodeMap;
use codemap::Span;
use codemap::Spanned;
use std::cmp::PartialEq;
use std::fmt::Debug;
use super::*;
use super::super::lexer;

fn test_parse<T: Debug + PartialEq, F, G>(source: &str, func: &F, expect: G)
where
  F: Fn(&mut ParseIter) -> Result<T, ParseErrorKind>,
  G: Fn(&File) -> Result<T, ParseErrorKind>,
{
  let mut map = CodeMap::new();
  let file = map.add_file(String::from("_test"), String::from(source));
  let tokens = lexer::lex(&file);
  let mut it = tokens.iter().peekable();

  assert_eq!(func(&mut it), expect(&file));

  assert_eq!(func(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();

  assert_eq!(func(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();

  assert_eq!(func(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_quark() {
  test_parse("null", &parse_quark, |_| Ok(Node::Null));
  test_parse("true", &parse_quark, |_| Ok(Node::Bool(true)));
  test_parse("false", &parse_quark, |_| Ok(Node::Bool(false)));
  test_parse("1.3", &parse_quark, |_| Ok(Node::Float(1.3)));
  test_parse("0.3", &parse_quark, |_| Ok(Node::Float(0.3)));
  test_parse("2", &parse_quark, |_| Ok(Node::Int(2)));
  test_parse("3", &parse_quark, |_| Ok(Node::Int(3)));
  test_parse("name", &parse_quark, |_| {
    Ok(Node::Name(String::from("name")))
  });
  test_parse("table", &parse_quark, |_| Ok(Node::Table));
}

#[test]
fn test_atom() {
  test_parse("null", &parse_atom, |_| Ok(Node::Null));

  test_parse("(null)", &parse_atom, |_| Ok(Node::Null));
}

#[test]
fn test_simple() {
  test_parse("foo", &parse_simple, |_| {
    Ok(Node::Name(String::from("foo")))
  });
  test_parse("foo.bar", &parse_simple, |_| {
    Ok(Node::Index {
      lhs: Box::new(Node::Name(String::from("foo"))),
      rhs: Box::new(Node::Str(String::from("bar"))),
    })
  });
  test_parse("foo[bar]", &parse_simple, |_| {
    Ok(Node::Index {
      lhs: Box::new(Node::Name(String::from("foo"))),
      rhs: Box::new(Node::Name(String::from("bar"))),
    })
  });
  test_parse("foo()", &parse_simple, |_| {
    Ok(Node::FuncCall {
      func: Box::new(Node::Name(String::from("foo"))),
      args: Vec::new(),
    })
  });
  test_parse("foo:bar()", &parse_simple, |_| {
    Ok(Node::MethodCall {
      owner: Box::new(Node::Name(String::from("foo"))),
      method: Box::new(Node::Str(String::from("bar"))),
      args: Vec::new(),
    })
  });
  test_parse("foo.bar()", &parse_simple, |_| {
    Ok(Node::FuncCall {
      func: Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("foo"))),
        rhs: Box::new(Node::Str(String::from("bar"))),
      }),
      args: Vec::new(),
    })
  });
  test_parse("foo.bar[baz]:qux()", &parse_simple, |_| {
    Ok(Node::MethodCall {
      owner: Box::new(Node::Index {
        lhs: Box::new(Node::Index {
          lhs: Box::new(Node::Name(String::from("foo"))),
          rhs: Box::new(Node::Str(String::from("bar"))),
        }),
        rhs: Box::new(Node::Name(String::from("baz"))),
      }),
      method: Box::new(Node::Str(String::from("qux"))),
      args: Vec::new(),
    })
  });
}

#[test]
fn test_fn_args() {
  test_parse("()", &parse_fn_args, |_| Ok(Vec::new()));
  test_parse("(x)", &parse_fn_args, |_| {
    Ok(vec![Node::Name(String::from("x"))])
  });
  test_parse("(x,)", &parse_fn_args, |_| {
    Ok(vec![Node::Name(String::from("x"))])
  });
  test_parse("(x,y)", &parse_fn_args, |_| {
    Ok(vec![
      Node::Name(String::from("x")),
      Node::Name(String::from("y")),
    ])
  });
  test_parse("(x,y,)", &parse_fn_args, |_| {
    Ok(vec![
      Node::Name(String::from("x")),
      Node::Name(String::from("y")),
    ])
  });
}

#[test]
fn test_un_expr() {
  test_parse("5", &parse_un_expr, |_| Ok(Node::Int(5)));

  test_parse("foo()", &parse_un_expr, |_| {
    Ok(Node::FuncCall {
      func: Box::new(Node::Name(String::from("foo"))),
      args: Vec::new(),
    })
  });

  test_parse("-5", &parse_un_expr, |_| {
    Ok(Node::UnExpr {
      op: lexer::Token::Sub,
      val: Box::new(Node::Int(5)),
    })
  });

  test_parse("-foo.bar", &parse_un_expr, |_| {
    Ok(Node::UnExpr {
      op: lexer::Token::Sub,
      val: Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("foo"))),
        rhs: Box::new(Node::Str(String::from("bar"))),
      }),
    })
  });

  test_parse("!-5", &parse_un_expr, |_| {
    Ok(Node::UnExpr {
      op: lexer::Token::Not,
      val: Box::new(Node::UnExpr {
        op: lexer::Token::Sub,
        val: Box::new(Node::Int(5)),
      }),
    })
  });
}

#[test]
fn test_bin_expr() {
  test_parse("1 + 2", &parse_bin_expr, |_| {
    Ok(Node::BinExpr {
      lhs: Box::new(Node::Int(1)),
      op: lexer::Token::Add,
      rhs: Box::new(Node::Int(2)),
    })
  });

  test_parse("(1 + 2)", &parse_bin_expr, |_| {
    Ok(Node::BinExpr {
      lhs: Box::new(Node::Int(1)),
      op: lexer::Token::Add,
      rhs: Box::new(Node::Int(2)),
    })
  });

  test_parse("1 + 2 * 3", &parse_bin_expr, |_| {
    Ok(Node::BinExpr {
      lhs: Box::new(Node::Int(1)),
      op: lexer::Token::Add,
      rhs: Box::new(Node::BinExpr {
        lhs: Box::new(Node::Int(2)),
        op: lexer::Token::Mul,
        rhs: Box::new(Node::Int(3)),
      }),
    })
  });
}

#[test]
fn test_fn_expr() {
  test_parse("|| 5", &parse_il_expr, |_| {
    Ok(Node::Lambda {
      params: Vec::new(),
      expr: Box::new(Node::Int(5)),
    })
  });

  test_parse("|x| 5", &parse_il_expr, |_| {
    Ok(Node::Lambda {
      params: vec![String::from("x")],
      expr: Box::new(Node::Int(5)),
    })
  });

  test_parse("|x,| 5", &parse_il_expr, |_| {
    Ok(Node::Lambda {
      params: vec![String::from("x")],
      expr: Box::new(Node::Int(5)),
    })
  });

  test_parse("|x,y| 5", &parse_il_expr, |_| {
    Ok(Node::Lambda {
      params: vec![String::from("x"), String::from("y")],
      expr: Box::new(Node::Int(5)),
    })
  });
}

#[test]
fn test_decl() {
  test_parse("x", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Single(String::from("x")),
      span: file.span.subspan(0, 1),
    })
  });

  test_parse("[x]", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Multi(vec![
        Spanned {
          node: Var::Single(String::from("x")),
          span: file.span.subspan(1, 2),
        },
      ]),
      span: file.span.subspan(0, 3),
    })
  });

  test_parse("[x, y]", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Multi(vec![
        Spanned {
          node: Var::Single(String::from("x")),
          span: file.span.subspan(1, 2),
        },
        Spanned {
          node: Var::Single(String::from("y")),
          span: file.span.subspan(4, 5),
        },
      ]),
      span: file.span.subspan(0, 6),
    })
  });

  test_parse("[[x, y], z]", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Multi(vec![
        Spanned {
          node: Var::Multi(vec![
            Spanned {
              node: Var::Single(String::from("x")),
              span: file.span.subspan(2, 3),
            },
            Spanned {
              node: Var::Single(String::from("y")),
              span: file.span.subspan(5, 6),
            },
          ]),
          span: file.span.subspan(1, 7),
        },
        Spanned {
          node: Var::Single(String::from("z")),
          span: file.span.subspan(9, 10),
        },
      ]),
      span: file.span.subspan(0, 11),
    })
  });

  test_parse("[x, [y, z]]", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Multi(vec![
        Spanned {
          node: Var::Single(String::from("x")),
          span: file.span.subspan(1, 2),
        },
        Spanned {
          node: Var::Multi(vec![
            Spanned {
              node: Var::Single(String::from("y")),
              span: file.span.subspan(5, 6),
            },
            Spanned {
              node: Var::Single(String::from("z")),
              span: file.span.subspan(8, 9),
            },
          ]),
          span: file.span.subspan(4, 10),
        },
      ]),
      span: file.span.subspan(0, 11),
    })
  });

  test_parse("[[x], [y]]", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Multi(vec![
        Spanned {
          node: Var::Multi(vec![
            Spanned {
              node: Var::Single(String::from("x")),
              span: file.span.subspan(2, 3),
            },
          ]),
          span: file.span.subspan(1, 4),
        },
        Spanned {
          node: Var::Multi(vec![
            Spanned {
              node: Var::Single(String::from("y")),
              span: file.span.subspan(7, 8),
            },
          ]),
          span: file.span.subspan(6, 9),
        },
      ]),
      span: file.span.subspan(0, 10),
    })
  });

  test_parse("[x, [y, z], q]", &parse_decl, |file| {
    Ok(Spanned {
      node: Var::Multi(vec![
        Spanned {
          node: Var::Single(String::from("x")),
          span: file.span.subspan(1, 2),
        },
        Spanned {
          node: Var::Multi(vec![
            Spanned {
              node: Var::Single(String::from("y")),
              span: file.span.subspan(5, 6),
            },
            Spanned {
              node: Var::Single(String::from("z")),
              span: file.span.subspan(8, 9),
            },
          ]),
          span: file.span.subspan(4, 10),
        },
        Spanned {
          node: Var::Single(String::from("q")),
          span: file.span.subspan(12, 13),
        },
      ]),
      span: file.span.subspan(0, 14),
    })
  });
}

#[test]
fn test_return_stmt() {
  test_parse("return", &parse_stmt, |_| Ok(Node::Return(None)));

  test_parse("return 5", &parse_stmt, |_| {
    Ok(Node::Return(Some(Box::new(Node::Int(5)))))
  });

  test_parse(
    "return fn()
       return 5",
    &parse_stmt,
    |_| {
      Ok(Node::Return(Some(Box::new(Node::FuncDef {
        params: Vec::new(),
        body: Box::new(Node::Block(vec![
          Node::Return(Some(Box::new(Node::Int(5)))),
        ])),
      }))))
    },
  );
}

#[test]
fn test_if_stmt() {
  test_parse(
    "if true
       pass",
    &parse_stmt,
    |_| {
      Ok(Node::If {
        cond: Box::new(Node::Bool(true)),
        body: Box::new(Node::Block(vec![Node::Pass])),
        els: None,
      })
    },
  );

  test_parse(
    "else if true
       pass",
    &parse_stmt,
    |_| {
      Ok(Node::ElseIf {
        cond: Box::new(Node::Bool(true)),
        body: Box::new(Node::Block(vec![Node::Pass])),
      })
    },
  );

  test_parse(
    "else
       pass",
    &parse_stmt,
    |_| {
      Ok(Node::Else {
        body: Box::new(Node::Block(vec![Node::Pass])),
      })
    },
  );
}

#[test]
fn test_for_stmt() {
  test_parse(
    "for x in true
       pass",
    &parse_stmt,
    |file| {
      Ok(Node::For {
        decl: Spanned {
          node: Var::Single(String::from("x")),
          span: file.span.subspan(4, 5),
        },
        expr: Box::new(Node::Bool(true)),
        body: Box::new(Node::Block(vec![Node::Pass])),
      })
    },
  );
}

#[test]
fn test_while_stmt() {
  test_parse(
    "while true
       pass",
    &parse_stmt,
    |_| {
      Ok(Node::While {
        expr: Box::new(Node::Bool(true)),
        body: Box::new(Node::Block(vec![Node::Pass])),
      })
    },
  );
}

#[test]
fn test_loop_stmt() {
  test_parse(
    "loop
       pass",
    &parse_stmt,
    |_| {
      Ok(Node::Loop {
        body: Box::new(Node::Block(vec![Node::Pass])),
      })
    },
  );
}

#[test]
fn test_place() {
  test_parse("x", &parse_place, |_| {
    Ok(Place::Single(Box::new(Node::Name(String::from("x")))))
  });

  test_parse("[x]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
    ]))
  });

  test_parse("[x.y]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("x"))),
        rhs: Box::new(Node::Str(String::from("y"))),
      })),
    ]))
  });

  test_parse("[x, y]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
      Place::Single(Box::new(Node::Name(String::from("y")))),
    ]))
  });

  test_parse("[[x, y], z]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Multi(vec![
        Place::Single(Box::new(Node::Name(String::from("x")))),
        Place::Single(Box::new(Node::Name(String::from("y")))),
      ]),
      Place::Single(Box::new(Node::Name(String::from("z")))),
    ]))
  });

  test_parse("[x, [y, z]]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
      Place::Multi(vec![
        Place::Single(Box::new(Node::Name(String::from("y")))),
        Place::Single(Box::new(Node::Name(String::from("z")))),
      ]),
    ]))
  });

  test_parse("[[x], [y]]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Multi(vec![Place::Single(Box::new(Node::Name(String::from("x"))))]),
      Place::Multi(vec![Place::Single(Box::new(Node::Name(String::from("y"))))]),
    ]))
  });

  test_parse("[x, [y, z], q]", &parse_place, |_| {
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
      Place::Multi(vec![
        Place::Single(Box::new(Node::Name(String::from("y")))),
        Place::Single(Box::new(Node::Name(String::from("z")))),
      ]),
      Place::Single(Box::new(Node::Name(String::from("q")))),
    ]))
  });
}
