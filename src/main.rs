extern crate clap;
extern crate codemap;
extern crate mask;
extern crate serde_yaml;

use clap::App;
use clap::Arg;
use codemap::CodeMap;
use mask::lexer::Token;
use mask::lexer;
use mask::parser::ParseErrorKind;
use mask::parser;
use mask::semck;
use std::fs::File;
use std::io::Write;
use std::io::prelude::*;
use std::io;
use std::path::Path;

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
        let span = map.look_up_span(token.span);
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

  let mut map = CodeMap::new();

  if let Some(source) = argv.value_of("code") {
    let file = map.add_file(String::from("_stdin"), source.to_string());

    // FIXME this code is duplicated a lot, but that's because there's no
    // "module" component in the compiler yet
    let tokens = lexer::lex(&file);
    let mut ast = match parser::parse(tokens) {
      Ok(root) => root,
      Err(why) => panic!("Couldn't semck: {:?}", why),
    };

    let mut ck = semck::SemChecker::new();
    match ck.check(&mut ast) {
      Err(why) => panic!("Bad semck: {:?}", why),
      _ => {}
    }

    println!("Checked: {}", serde_yaml::to_string(&ast).unwrap());
  } else if let Some(filename) = argv.value_of("path") {
    let path = Path::new(&filename);
    let mut file = match File::open(path) {
      Ok(file) => file,
      Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
    };

    let mut contents = String::new();
    let cm_file = match file.read_to_string(&mut contents) {
      Ok(_) => map.add_file(filename.to_string(), contents.to_string()),
      Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
    };

    // see FIXME above
    let tokens = lexer::lex(&cm_file);
    let mut ast = match parser::parse(tokens) {
      Ok(root) => root,
      Err(why) => panic!("Couldn't semck: {:?}", why),
    };

    let mut ck = semck::SemChecker::new();
    match ck.check(&mut ast) {
      Err(why) => panic!("Bad semck: {:?}", why),
      _ => {}
    }

    println!("Checked: {}", serde_yaml::to_string(&ast).unwrap());
  } else {
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

          let file = map.add_file(String::from("_stdin"), chunk.clone());

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
