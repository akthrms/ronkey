use ronkey::repl;
use std::io;
use whoami;

fn main() -> io::Result<()> {
    let username = whoami::username();
    println!(
        "Hello {}! This is the Monkey programming language!",
        username
    );
    println!("Feel free to type in commands");

    repl::start()
}
