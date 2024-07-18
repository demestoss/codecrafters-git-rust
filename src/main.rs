#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    process(args)
}

fn process(args: Vec<String>) {
    match args[1].as_str() {
        "init" => handle_init_command(),
        _ => println!("unknown command: {}", args[1]),
    }
}

fn handle_init_command() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}
