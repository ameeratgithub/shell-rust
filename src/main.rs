#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::HashSet, sync::LazyLock};

static KEYWORDS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("echo");
    set.insert("exit");
    set.insert("type");
    set
});

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();
        if command == "exit" {
            break;
        }

        let args = command.split_whitespace().collect::<Vec<&str>>();
        let command = args[0];

        match command {
            "echo" => {
                let output = &args[1..].join(" ");
                println!("{output}");
            }
            "exit" => {
                break;
            }
            "type" => {
                if KEYWORDS.contains(args[1]) {
                    println!("{} is a shell builtin", args[1])
                } else {
                    println!("{}: not found", args[1]);
                }
            }
            _ => {
                println!("{command}: command not found");
            }
        }
    }
}
