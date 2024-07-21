mod commands;
mod objects;
use crate::commands::cat_file::CatObjectFlags;
use crate::commands::{cat_file, hash_object, init};
use clap::{ArgGroup, Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    #[clap(group(ArgGroup::new("info").required(true).args(&["object_content", "object_exists", "object_type", "object_size"])))]
    CatFile {
        #[clap(short = 'p')]
        object_content: bool,
        #[clap(short = 'e')]
        object_exists: bool,
        #[clap(short = 't')]
        object_type: bool,
        #[clap(short = 's')]
        object_size: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    process(&args)
}

fn process(args: &[String]) -> anyhow::Result<()> {
    let args = Args::parse_from(args);

    match args.command {
        Command::Init => init::handle()?,
        Command::CatFile {
            object_size,
            object_hash,
            object_exists,
            object_type,
            object_content,
        } => cat_file::handle(
            &object_hash,
            CatObjectFlags {
                object_size,
                object_exists,
                object_type,
                object_content,
            },
        ),
        Command::HashObject { write, file } => hash_object::handle(&file, write)?,
    };
    Ok(())
}
