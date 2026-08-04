use ffi;

use crate::core::{Allocator, Version, Result};

use std::io::{Read, Write, Seek};
use std::path::PathBuf;

use bitflags::bitflags;

pub struct FileInfo {
    exists: bool,
    is_directory: bool,
    size: u64,
    created_time: u64,
    modified_time: u64,
    accessed_time: u64,
}

pub struct File {
    sys: ffi::emplat_file,
}

impl File {
    pub fn open(filepath: &str, flags: FileFlags) -> Result<File> {
        todo!()
    }

    pub fn info(filepath: &str) -> FileInfo {

    }
    
    pub fn size(&self) -> u64 {

    }

    pub fn lock(&mut self) -> Result<()> {

    }

    pub fn unlock(&mut self) -> Result<()> {

    }

    pub fn write_safe(&mut self, buffer: &[u8]) -> Result<usize> {

    }
}

impl Drop for File {

}

impl Read for File {

}

impl Write for File {

}

impl Seek for File {

}

pub enum FileWatchEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed(PathBuf, PathBuf),
}

pub struct FileWatcher {
    sys: ffi::emplat_filewatcher,
}

impl FileWatcher {
    pub fn create() -> Result<FileWatcher> {

    }

    pub fn add(&mut self, filepath: &[&PathBuf]) -> Result<()> {

    }

    pub fn add_pattern(&mut self, pattern: &str) -> Result<()> {

    }

    pub fn poll_files(&mut self) -> Option<FileWatchEvent> {

    }
}

impl Drop for FileWatcher {

}
