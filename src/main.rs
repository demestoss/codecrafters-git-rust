mod commands;
mod objects;
use crate::commands::{cat_file, hash_object, init};
use std::env;
use std::io::{Read, Write};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    process(&args[1..])
}

fn process(args: &[String]) -> anyhow::Result<()> {
    match args[0].as_str() {
        "init" => init::handle()?,
        "cat-file" => cat_file::handle(&args[1..]),
        "hash-object" => hash_object::handle(&args[1..])?,
        _ => println!("unknown command: {}", args[1]),
    };
    Ok(())
}
