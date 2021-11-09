use std::fs::{File, OpenOptions};
use std::io::{BufReader, Error as IoError, Read, Write};
use std::path::Path;

use crate::parser;

#[derive(Debug, Copy, Clone)]
pub struct HunkPos {
    pub start: usize,
    pub end: usize,
}

impl HunkPos {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct Hunk {
    name: String,
    info: Option<String>,
    data: Vec<u8>,
    original_pos: HunkPos,
}

impl Hunk {
    pub(crate) fn new(name: String, info: Option<String>, data: Vec<u8>, pos: HunkPos) -> Self {
        Self {
            name,
            info,
            data,
            original_pos: pos,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into();
    }

    pub fn info(&self) -> Option<&String> {
        self.info.as_ref()
    }

    pub fn set_info(&mut self, info: Option<String>) {
        self.info = info;
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_string(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn original_pos(&self) -> HunkPos {
        self.original_pos
    }

    fn render(&self) -> Vec<u8> {
        let mut r = Vec::new();
        r.extend_from_slice(
            &format!(
                "[testmark]:# ({})\n```{}\n",
                self.name,
                &self.info.as_ref().map(|i| i.as_str()).unwrap_or("")
            )
            .as_bytes(),
        );
        r.extend_from_slice(&self.data);
        r.extend_from_slice(b"\n```");
        r
    }
}

#[derive(Debug)]
pub struct Document {
    pub body: Vec<u8>,
    pub hunks: Vec<Hunk>,
}

impl Document {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Document, IoError> {
        let f = File::open(path.as_ref())?;
        let mut reader = BufReader::new(f);
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes)?;

        Ok(parser::create_document(bytes))
    }

    pub fn from_string(contents: &str) -> Document {
        parser::create_document(contents.as_bytes().to_owned())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Document {
        parser::create_document(bytes)
    }

    pub fn hunks(&self) -> &[Hunk] {
        &self.hunks
    }

    pub fn hunks_mut(&mut self) -> &mut [Hunk] {
        &mut self.hunks
    }

    pub fn render(&self) -> Vec<u8> {
        let mut new_doc: Vec<u8> = Vec::new();
        let mut doc_cursor = 0;
        for hunk in self.hunks() {
            let section_start = hunk.original_pos().start;
            let section_end = hunk.original_pos().end;

            new_doc.extend_from_slice(&self.body[doc_cursor..section_start]);
            new_doc.extend_from_slice(&hunk.render());
            doc_cursor = section_end;
        }
        if doc_cursor < self.body.len() {
            new_doc.extend_from_slice(&self.body[doc_cursor..self.body.len()]);
        }
        new_doc
    }

    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<(), IoError> {
        let doc = self.render();
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.as_ref())?;
        f.write_all(&doc)?;
        Ok(())
    }
}
