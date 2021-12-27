use crate::GitError;
use std::{fs::{File, read_dir}, path::Path, io::Write};


pub fn is_dir_empty(path: &Path) -> bool {
    read_dir(&path).unwrap().map(|l| 1).sum::<i32>() ==0
}

pub fn create_write_file(path: &Path, contents: &str) -> Result<(), GitError> {
    let mut file = File::create(path).map_err(|_e| {
        GitError::PathError("Could not create file".to_owned(), path.to_path_buf())
    })?;

    file.write_all(contents.as_bytes()).map_err(|_e| {
        GitError::PathError("Could not write file".to_owned(), path.to_path_buf())
    })?;

    file.write(b"\n").map_err(|e| e.to_string()).map_err(|_e| {
        GitError::PathError("Could not write file".to_owned(), path.to_path_buf())
    })?;

    Ok(())
}