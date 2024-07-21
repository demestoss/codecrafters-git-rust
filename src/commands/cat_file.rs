use crate::objects::Object;
use std::io;

pub struct CatObjectFlags {
    pub object_content: bool,
    pub object_exists: bool,
    pub object_type: bool,
    pub object_size: bool,
}

pub fn handle(object_hash: &str, flags: CatObjectFlags) -> anyhow::Result<()> {
    let mut object = Object::read(object_hash)?;

    match flags {
        CatObjectFlags {
            object_content: true,
            ..
        } => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            io::copy(&mut object.reader, &mut stdout)?;
        }
        CatObjectFlags {
            object_exists: true,
            ..
        } => print!("object exists!"),
        CatObjectFlags {
            object_type: true, ..
        } => print!("{}", object.kind),
        CatObjectFlags {
            object_size: true, ..
        } => print!("{}", object.size),
        _ => println!("usage: git cat-file (-p | -e | -t | -s) <object>"),
    };
    Ok(())
}
