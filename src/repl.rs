use crate::evaluator::evaluate;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::io;
use std::io::Write;

pub fn start() -> io::Result<()> {
    loop {
        print!(">> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        let mut lexer = Lexer::new(&line);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        if parser.exists_errors() {
            print_parse_errors(parser.get_errors())?;
            continue;
        }

        match evaluate(program) {
            Ok(evaluated) => println!("{}", evaluated),
            Err(message) => println!("ERROR: {}", message),
        };

        io::stdout().flush()?;
    }
}

const MONKEY_FACE: &str = r#"
           __,__
  .--.  .-"     "-.  .--.
 / .. \/  .-. .-.  \/ .. \
| |  '|  /   Y   \  |'  | |
| \   \  \ 0 | 0 /  /   / |
 \ '- ,\.-"""""""-./, -' /
  ''-' /_   ^ ^   _\ '-''
      |  \._   _./  |
      \   \ '~' /   /
       '._ '-=-' _.'
          '-----'
"#;

fn print_parse_errors(errors: Vec<String>) -> io::Result<()> {
    println!("{}", MONKEY_FACE);
    println!("Woops! We ran into some monkey business here!");
    println!(" parser errors:");

    for error in errors {
        println!("\t{}", error);
        io::stdout().flush()?;
    }

    Ok(())
}