use std::io::Write;

pub enum LoadingFileSource {
    InMemory(Vec<u8>),
    OnDisk(std::io::BufWriter<std::fs::File>)
}

impl LoadingFileSource {
    pub fn new(file_len: Option<u64>) -> Result<LoadingFileSource, anyhow::Error> {
        Ok(match file_len {
            Some(x) => {
                if x < 1024*1024*1024 {
                    let contents = Vec::with_capacity(x.try_into().unwrap());
                    println!("In-Memory File");
                    LoadingFileSource::InMemory(contents)
                } else {
                    println!("Disk File");
                    LoadingFileSource::OnDisk(std::io::BufWriter::new(tempfile::tempfile()?))
                }
            }
            _ => {
                println!("Disk File");
                LoadingFileSource::OnDisk(std::io::BufWriter::new(tempfile::tempfile()?))
            }
        })
    }

    pub fn add(&mut self, slice: &[u8]) -> Result<(), anyhow::Error> {
        match self {
            LoadingFileSource::InMemory(v) => {
                v.extend_from_slice(slice);
                Ok(())
            },
            LoadingFileSource::OnDisk(f) => {
                f.write_all(slice).map_err(|x|x.into())
            }
        }
    }
}

pub enum LoadedFileSource {
    InMemory(Vec<u8>),
    OnDisk(std::fs::File)
}

pub trait BufReadSeek: std::io::BufRead + std::io::Seek {}
impl<T: std::io::BufRead + std::io::Seek> BufReadSeek for T {}

impl LoadedFileSource {
    pub fn new(loading: LoadingFileSource) -> LoadedFileSource {
        match loading {
            LoadingFileSource::InMemory(v) => LoadedFileSource::InMemory(v),
            LoadingFileSource::OnDisk(f) => LoadedFileSource::OnDisk(f.into_inner().unwrap())
        }
    } 
    
    pub fn as_reader(self) -> Box<dyn BufReadSeek> {
        match self {
            LoadedFileSource::InMemory(v) => {
                Box::new(std::io::BufReader::new(std::io::Cursor::new(v)))
            },
            LoadedFileSource::OnDisk(f) => {
                Box::new(std::io::BufReader::new(f))
            }
        }
    }
}
