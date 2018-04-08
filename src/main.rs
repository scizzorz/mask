extern crate clap;
extern crate codemap;
extern crate mask;

use clap::App;
use clap::Arg;
use codemap::CodeMap;
use std::fs::File;
use std::io::Write;
use std::io::prelude::*;
use std::io;
use std::path::Path;

fn print_tokens(map: &CodeMap, tokens: &Vec<codemap::Spanned<mask::lexer::Token>>) {
  let mut indent = 0;

  for token in tokens {
    match token.node {
      mask::lexer::Token::End | mask::lexer::Token::EOF => {
        println!("{:?}", token.node);
        for _ in 0..indent {
          print!(" ");
        }
      }

      mask::lexer::Token::Enter => {
        indent += 2;
        println!("{:?}", token.node);
        for _ in 0..indent {
          print!(" ");
        }
      }

      mask::lexer::Token::Exit => {
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
    let tokens = mask::lexer::lex(&file);
    let ast = mask::parser::parse(tokens);
    println!("AST: {:?}", ast);
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
    let tokens = mask::lexer::lex(&cm_file);
    let ast = mask::parser::parse(tokens);
    println!("AST: {:?}", ast);
  } else {
    println!("Starting REPL");
    loop {
      let mut buffer = String::new();
      print!("> ");
      io::stdout().flush();

      match io::stdin().read_line(&mut buffer) {
        Ok(nbytes) => {
          if nbytes == 0 || buffer == "quit\n" {
            println!("");
            break;
          }

          let file = map.add_file(String::from("_stdin"), buffer);

          // see FIXME above
          let tokens = mask::lexer::lex(&file);
          let ast = mask::parser::parse(tokens);
          println!("AST: {:?}", ast);
        }

        Err(why) => {
          panic!("Unable to read line: {}", why);
        }
      }
    }
  }
}
