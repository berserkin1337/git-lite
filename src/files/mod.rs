use crate::GitError;
use std::env;
use std::io::Read;
use std::{
    fs::{read_dir, File},
    io::Write,
    path::{Path, PathBuf},
};

pub fn is_dir_empty(path: &Path) -> bool {
    read_dir(&path).unwrap().map(|_l| 1).sum::<i32>() == 0
}

pub fn create_write_file(path: &Path, contents: &str) -> Result<(), GitError> {
    let mut file = File::create(path).map_err(|_e| {
        GitError::PathError("Could not create file".to_owned(), path.to_path_buf())
    })?;

    file.write_all(contents.as_bytes())
        .map_err(|_e| GitError::PathError("Could not write file".to_owned(), path.to_path_buf()))?;

    file.write(b"\n")
        .map_err(|e| e.to_string())
        .map_err(|_e| GitError::PathError("Could not write file".to_owned(), path.to_path_buf()))?;

    Ok(())
}

pub fn cwd() -> Result<PathBuf, GitError> {
    env::current_dir()
        .map_err(|_| GitError::GenericError("Cannot open current working directory!".to_owned()))
}

pub fn read_data(path: &Path) -> Result<Vec<u8>, GitError> {
    let mut data = Vec::new();

    File::open(&path)
        .and_then(|mut file| file.read_to_end(&mut data))
        .and(Ok(data))
        .map_err(|e| {
            GitError::PathError(
                format!("Could not read file {}", e.to_string()),
                path.to_path_buf(),
            )
        })
}
