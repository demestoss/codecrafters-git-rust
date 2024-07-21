use crate::objects;

pub struct CatObjectFlags {
    pub object_content: bool,
    pub object_exists: bool,
    pub object_type: bool,
    pub object_size: bool,
}

pub fn handle(object_hash: &str, flags: CatObjectFlags) {
    let result = match flags {
        CatObjectFlags {
            object_content: true,
            ..
        } => display_object_info(object_hash, get_object_content),
        CatObjectFlags {
            object_exists: true,
            ..
        } => display_object_info(object_hash, get_object_exists),
        CatObjectFlags {
            object_type: true, ..
        } => display_object_info(object_hash, get_object_type),
        CatObjectFlags {
            object_size: true, ..
        } => display_object_info(object_hash, get_object_size),
        _ => {
            println!("usage: git cat-file (-p | -e | -t | -s) <object>");
            Ok(())
        }
    };

    match result {
        Err(e) => eprintln!("incorrect object: {e}"),
        _ => {}
    }
}

fn display_object_info(
    object_hash: &str,
    getter_fn: fn(object: &str) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let object_content = objects::get_object_by_hash(object_hash)?;
    let object = objects::decompress(&object_content)?;
    let info = getter_fn(&object)?;
    print!("{info}");
    Ok(())
}

fn get_object_exists(_: &str) -> anyhow::Result<String> {
    Ok("object exists!".to_owned())
}

fn get_object_type(object: &str) -> anyhow::Result<String> {
    Ok(object.split(' ').take(1).collect())
}

fn get_object_size(object: &str) -> anyhow::Result<String> {
    Ok(object
        .chars()
        .skip_while(|&v| v != ' ')
        .skip(1)
        .take_while(|&v| v != char::from(0x00))
        .collect())
}

fn get_object_content(object: &str) -> anyhow::Result<String> {
    Ok(object
        .chars()
        .skip_while(|&v| v != char::from(0x00))
        .skip(1)
        .collect())
}
