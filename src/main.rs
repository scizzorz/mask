extern crate clap;
extern crate codemap;
extern crate mask;

use clap::App;
use clap::Arg;
use codemap::CodeMap;
use mask::engine;
use mask::lexer::Token;
use mask::lexer;
use mask::module;
use mask::parser::ParseErrorKind;
use mask::parser;
use std::io::Write;
use std::io::prelude::*;
use std::io;

fn print_tokens(map: &CodeMap, tokens: &Vec<codemap::Spanned<lexer::Token>>) {
  let mut indent = 0;

  for token in tokens {
    match token.node {
      lexer::Token::End | lexer::Token::EOF => {
        println!("{:?}", token.node);
        for _ in 0..indent {
          print!(" ");
        }
      }

      lexer::Token::Enter => {
        indent += 2;
        println!("{:?}", token.node);
        for _ in 0..indent {
          print!(" ");
        }
      }

      lexer::Token::Exit => {
        indent -= 2;
        print!("{:?} ", token.node);
      }

      _ => {
        let _span = map.look_up_span(token.span);
        print!("{:?} ", token.node);
        //print!("{}:{}:{}: {:?} ", span.file.name(), span.begin.line, span.begin.column, token.node);
        //print!("{:?}<{:?}> ", token.node, );
      }
    }
  }
}

fn main() {
  let argv = App::new("mask")
    .version("0.0.1")
    .author("John Weachock <jweachock@gmail.com>")
    .about("A programming language.")
    .arg(
      Arg::with_name("code")
        .short("c")
        .long("code")
        .value_name("CODE")
        .help("Mask code to execute. Overrides any specified module")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("path")
        .takes_value(true)
        .index(1)
        .help("Mask module to execute"),
    )
    .get_matches();

  let mut engine = engine::Engine::new();

  if let Some(source) = argv.value_of("code") {
    let mut module = module::Module::from_string(&mut engine.map, source);

    match module {
      Ok(module) => {
        println!("Constants:");
        for data in module.consts {
          println!("{:?}", data);
        }

        println!("Code:");
        for instr in module.code {
          println!("{:?}", instr);
        }
      }
      Err(why) => panic!("Unable to check: {:?}", why),
    }
  } else if let Some(filename) = argv.value_of("path") {
    let mut module = module::Module::from_file(&mut engine.map, filename);

    match module {
      Ok(module) => {
        println!("Constants:");
        for data in module.consts {
          println!("{:?}", data);
        }

        println!("Code:");
        for instr in module.code {
          println!("{:?}", instr);
        }
      }
      Err(why) => panic!("Unable to check: {:?}", why),
    }
  } else {
    // FIXME this is a nightmare
    // FIXME needs to handle multiline statements
    // initial idea is to request an extra line when the AST matches
    // UnexpectedToken(End) | UnexpectedEOF
    // and then concatenate it to the previous line(s)
    // FIXME ^ this is implemented, but isn't quite right
    // ie, if you enter `if` as your first line, it will never get it right
    // there needs to be an 'unexpected X, expected Token::Enter`, maybe?

    let mut chunk = String::new();
    let mut wait_for_blank = false;

    loop {
      let mut buffer = String::new();
      if wait_for_blank {
        print!(". ");
      } else {
        print!("> ");
      }
      io::stdout().flush();

      match io::stdin().read_line(&mut buffer) {
        Ok(nbytes) => {
          chunk.push_str(&buffer);

          if nbytes == 0 || chunk == "quit\n" {
            println!("");
            break;
          }

          if wait_for_blank && buffer != "\n" {
            continue;
          }

          let file = engine.map.add_file(String::from("_stdin"), chunk.clone());

          let tokens = lexer::lex(&file);
          let ast = parser::parse(tokens);
          match ast {
            // incomplete statement - say we're waiting for an empty line and then skip the rest
            Err(ParseErrorKind::UnexpectedToken(Token::End))
            | Err(ParseErrorKind::UnexpectedEOF) => {
              wait_for_blank = true;
              continue;
            }

            // we have a complete statement! parse it!
            _ => {
              println!("AST: {:?}", ast);
              chunk.clear();
              wait_for_blank = false;
            }
          }
        }

        Err(why) => {
          panic!("Unable to read line: {}", why);
        }
      }
    }
  }
}
