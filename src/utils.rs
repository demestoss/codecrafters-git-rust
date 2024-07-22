use anyhow::Context;

pub fn from_bytes_with_nul(buf: &[u8]) -> anyhow::Result<String> {
    let str = String::from_utf8(buf[..buf.len() - 1].to_owned())
        .context("parse string with the last \0 symbols")?;
    Ok(str)
}
