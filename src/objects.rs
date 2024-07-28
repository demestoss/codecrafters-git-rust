use crate::utils::from_bytes_with_nul;
use anyhow::{bail, Context};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fmt::{Display, Formatter};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

pub fn get_object_dir_path(hash: &str) -> PathBuf {
    Path::new(".git/objects").join(&hash[..2]).to_owned()
}

pub fn get_object_path(hash: &str) -> PathBuf {
    let dir = get_object_dir_path(hash);
    dir.join(&hash[2..])
}

#[derive(Debug, PartialEq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl FromStr for ObjectKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectKind::Blob),
            "tree" => Ok(ObjectKind::Tree),
            "commit" => Ok(ObjectKind::Commit),
            _ => bail!("unknown object type: {s}"),
        }
    }
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Tree => write!(f, "tree"),
            ObjectKind::Blob => write!(f, "blob"),
            ObjectKind::Commit => write!(f, "commit"),
        }
    }
}

pub type ObjectHash = [u8; 20];

pub struct Object<R> {
    pub kind: ObjectKind,
    pub size: u64,
    pub reader: R,
}

impl Object<()> {
    pub fn blob_from_file(file_name: &Path) -> anyhow::Result<Object<impl Read>> {
        let file_stat =
            fs::metadata(file_name).with_context(|| format!("stat {}", file_name.display()))?;
        let size = file_stat.len();

        let file =
            fs::File::open(file_name).with_context(|| format!("open {}", file_name.display()))?;

        return Ok(Object {
            kind: ObjectKind::Blob,
            size,
            reader: file,
        });
    }

    pub fn read_from_objects(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        if hash.len() != 40 {
            bail!("incorrect object hash {hash}");
        }

        let path = get_object_path(hash);
        let file = fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;

        let d = ZlibDecoder::new(file);
        let mut r = BufReader::new(d);
        let mut buf = Vec::new();
        r.read_until(0x00, &mut buf).context("read object header")?;

        let head = from_bytes_with_nul(&buf)?;
        let Some((kind, size)) = head.split_once(' ') else {
            bail!(".git/objects file head signature is incorrect '{head}'")
        };

        let kind = ObjectKind::from_str(kind)?;
        let size = size
            .parse::<u64>()
            .with_context(|| format!(".git/objects file head has invalid size {size}"))?;
        let r = r.take(size);

        Ok(Object {
            kind,
            size,
            reader: r,
        })
    }
}

impl<R: Read> Object<R> {
    pub fn write(mut self, writer: impl Write) -> anyhow::Result<ObjectHash> {
        let mut object_writer = ObjectWriter::new(writer);

        write!(object_writer, "{} {}\0", self.kind, self.size)?;
        io::copy(&mut self.reader, &mut object_writer)
            .context("stream object content into writer")?;

        let hash = object_writer.finalize();
        object_writer.writer.finish()?;

        Ok(hash)
    }

    pub fn write_to_objects(self) -> anyhow::Result<ObjectHash> {
        let tmp = "temporary";
        let hash = self
            .write(fs::File::create(tmp).context("construct temporary file for tree")?)
            .context("stream object content into in-memory buffer")?;
        let hash_hex = hex::encode(&hash);

        fs::create_dir_all(get_object_dir_path(&hash_hex))
            .context("create .git/objects directory")?;
        fs::rename(tmp, get_object_path(&hash_hex)).with_context(|| {
            format!(
                "stream object from tmp file to .git/object blob {}",
                get_object_path(&hash_hex).display()
            )
        })?;

        Ok(hash)
    }
}

pub struct ObjectWriter<W: Write> {
    hasher: Sha1,
    writer: ZlibEncoder<W>,
}

impl<W: Write> ObjectWriter<W> {
    pub fn new(w: W) -> Self {
        let writer = ZlibEncoder::new(w, Compression::default());
        let hasher = Sha1::new();
        ObjectWriter { writer, hasher }
    }

    pub fn finalize(&mut self) -> ObjectHash {
        let output = self.hasher.finalize_reset();
        output.into()
    }
}

impl<W: Write> Write for ObjectWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
