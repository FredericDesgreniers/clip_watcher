use std::fs::File;
use std::io::Error;
use std::io::Read;

pub trait FileExtension {
    fn read_as_string(&mut self) -> Result<String, Error>;
}

impl FileExtension for File {
    fn read_as_string(&mut self) -> Result<String, Error> {
        let mut content = String::new();
        let _ = self.read_to_string(&mut content)?;

        Ok(content)
    }
}
