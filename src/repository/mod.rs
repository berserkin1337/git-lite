pub mod config;

use configparser::ini::Ini;

use self::config::GitConfig;
use crate::{error::GitError, files};
use crate::files::is_dir_empty;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct GitRepository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub config: GitConfig,
}

impl GitRepository {
    // Computes a path under the repo's gitdir
    pub fn repo_path(&self, path: &Path) -> PathBuf {
        self.gitdir.join(path)
    }
    //
    pub fn repo_dir(&self, path: &Path, mkdir: bool) -> Option<PathBuf> {
        let path = self.repo_path(path);
        if path.exists() {
            if path.is_dir() {
                return Some(path.clone());
            } else {
                return None;
            }
        } else if mkdir {
            create_dir_all(&path).unwrap();
            return Some(path.clone());
        }
        None
    }
    pub fn repo_file(&self, path: &Path) -> Result<PathBuf, GitError> {
        let full_path = self.repo_path(path);
        let parent = full_path.parent().unwrap();
        create_dir_all(parent).map_err(|_| {
            GitError::PathError(
                "Could not create directories: {}".to_owned(),
                parent.to_path_buf(),
            )
        })?;
        Ok(full_path)
    }
    // Create a new git repository .
    pub fn new(path: &Path) -> GitRepository {
        let gitdir = path.join(&path!(".gitdir"));
        let conf = Ini::new();

        GitRepository {
            worktree: path.to_path_buf(),
            gitdir: gitdir,
            config: GitConfig::new(conf),
        }
    }

    // Write a new git repository to the path.

    pub fn write_to_path(path: &Path) -> Result<GitRepository, GitError> {
        let repo = GitRepository::new(path);
        GitRepository::create_dir(&repo, path)?;

        repo.repo_dir(&path!("branches"), true);
        repo.repo_dir(&path!("objects"), true);
        repo.repo_dir(&path!("refs/tags"), true);
        repo.repo_dir(&path!("refs/heads"), true);
        GitRepository::create_repo_file(
            &repo,
            &path!("description"),
            "Unnamed repository; edit this file 'description' to name the repository.",
        )?;
        GitRepository::create_repo_file(&repo, &path!("HEAD"), "ref: refs/heads/master")?;        let default_config: GitConfig = Default::default();
        let config = repo.repo_file(&path!("config"))?;
        default_config.save(&config).unwrap();

        Ok(repo)
    }

    // This function creates the repository 's base working directory.
    // It will throw an error if the directory already exists and it is not empty, or if the given path is not a directory.
    pub fn create_dir(repo: &GitRepository, path: &Path) -> Result<(), GitError> {
        if repo.worktree.exists() {
            if !repo.worktree.is_dir() {
                return Err(GitError::PathError(
                    "Specified directory is not empty".to_owned(),
                    repo.worktree.clone(),
                ));
            } else if !is_dir_empty(path) {
                return Err(GitError::PathError(
                    "Could not create directory".to_owned(),
                    repo.worktree.clone(),
                ));
            }
        } else {
            create_dir_all(&repo.worktree).map_err(|_| {
                GitError::PathError(
                    "Could not create directory".to_owned(),
                    repo.worktree.clone(),
                )
            })?;
        }
        Ok(())
    }

    fn create_repo_file(repo: &GitRepository, path: &Path, contents: &str) -> Result<(), GitError> {
        let file = repo.repo_file(path)?;
        files::create_write_file(&file, contents)
    }
}
