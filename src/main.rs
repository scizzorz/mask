extern crate rain;
extern crate clap;
extern crate codemap;

use clap::App;
use clap::Arg;
use codemap::CodeMap;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

fn print_tokens(map: &CodeMap, tokens: &Vec<codemap::Spanned<rain::lexer::Token>>) {
  let mut indent = 0;

  for token in tokens {
    match token.node {
      rain::lexer::Token::End | rain::lexer::Token::EOF => {
        println!("{:?}", token.node);
        for _ in 0..indent {
          print!(" ");
        }
      }

      rain::lexer::Token::Enter => {
        indent += 2;
        println!("{:?}", token.node);
        for _ in 0..indent {
          print!(" ");
        }
      }

      rain::lexer::Token::Exit => {
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
  let argv = App::new("rain")
    .version("0.0.1")
    .author("John Weachock <jweachock@gmail.com>")
    .about("A programming language.")
    .arg(Arg::with_name("code")
         .short("c")
         .long("code")
         .value_name("CODE")
         .help("Rain code to execute. Overrides any specified module")
         .takes_value(true))
    .arg(Arg::with_name("path")
         .takes_value(true)
         .index(1)
         .help("Rain module to execute"))
    .get_matches();

  let mut map = CodeMap::new();

  let file = match argv.value_of("code") {
    Some(x) => map.add_file(String::from("_stdin"), x.to_string()),
    None => {
      let path_name = argv.value_of("path").unwrap_or("test.rn");
      let path = Path::new(&path_name);
      let mut file = match File::open(path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
      };

      let mut contents = String::new();
      match file.read_to_string(&mut contents) {
        Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
        Ok(_) => map.add_file(path_name.to_string(), contents.to_string()),
      }
    }
  };

  let tokens = rain::lexer::lex(&file);
  print_tokens(&map, &tokens);
  /*
  let ast = rain::parser::parse(tokens);
  println!("AST: {:?}", ast);
  */
}
