use crate::commands::cat_file::CatObjectFlags;
use anyhow::{bail, Context};
use clap::{ArgGroup, Parser, Subcommand};
use std::fmt::format;
use std::path::{Path, PathBuf};
use std::{env, fs};

mod commands;
mod objects;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    #[clap(group(ArgGroup::new("info").required(true).args(&["pretty_print", "object_exists", "object_type", "object_size"])))]
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
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
    LsTree {
        #[clap(long = "name-only")]
        name_only: bool,

        object_hash: String,
    },
    WriteTree,
    CommitTree {
        #[clap(short = 'p', long = "parent")]
        parent_hash: Option<String>,
        #[clap(short = 'm', long = "message")]
        commit_message: String,

        tree_hash: String,
    },
    Commit {
        #[clap(short = 'm', long = "message")]
        commit_message: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    process(&args)
}

fn process(args: &[String]) -> anyhow::Result<()> {
    let args = Args::parse_from(args);

    match args.command {
        Command::Init => commands::init::handle()?,
        Command::CatFile {
            object_size,
            object_hash,
            object_exists,
            object_type,
            pretty_print,
        } => commands::cat_file::handle(
            &object_hash,
            CatObjectFlags {
                object_size,
                object_exists,
                object_type,
                pretty_print,
            },
        )?,
        Command::HashObject { write, file } => commands::hash_object::handle(&file, write)?,
        Command::LsTree {
            name_only,
            object_hash,
        } => commands::ls_tree::handle(&object_hash, name_only)?,
        Command::WriteTree => commands::write_tree::handle()?,
        Command::CommitTree {
            tree_hash,
            parent_hash,
            commit_message,
        } => commands::commit_tree::handle(tree_hash, parent_hash, commit_message)?,
        Command::Commit { commit_message } => {
            let head_ref = fs::read_to_string(".git/HEAD").context("read HEAD")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                bail!("refusing to commit onto detached HEAD");
            };
            let head_ref = head_ref.trim_end();
            let parent_hash = fs::read_to_string(format!(".git/{head_ref}"))
                .with_context(|| format!("read {head_ref} ref file"))?;
            let parent_hash = parent_hash.trim_end();

            let Some(tree_hash) = commands::write_tree::write_tree_for(Path::new("."))? else {
                bail!("not commiting empty tree")
            };
            let commit_hash = commands::commit_tree::write_commit(
                &hex::encode(tree_hash),
                Some(parent_hash),
                &commit_message,
            )
            .context("create commit")?;
            let commit_hash = hex::encode(commit_hash);

            fs::write(format!(".git/{head_ref}"), &commit_hash)
                .with_context(|| format!("update HEAD reference target {head_ref}"))?;

            println!("HEAD is now at commit {commit_hash}");
        }
    };
    Ok(())
}
