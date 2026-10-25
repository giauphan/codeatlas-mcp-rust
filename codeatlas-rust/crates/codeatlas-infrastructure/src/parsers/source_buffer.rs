use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub enum SourceBuffer {
    Mmap(Mmap),
    Heap(String),
}

impl SourceBuffer {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        // Use mmap for files > 16KB to eliminate heap allocation
        if metadata.len() > 16 * 1024 {
            let mmap = unsafe { Mmap::map(&file)? };
            Ok(Self::Mmap(mmap))
        } else {
            let s = std::fs::read_to_string(path)?;
            Ok(Self::Heap(s))
        }
    }

    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        match self {
            Self::Mmap(m) => std::str::from_utf8(m),
            Self::Heap(s) => Ok(s.as_str()),
        }
    }
}
