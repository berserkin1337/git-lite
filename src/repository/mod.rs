pub mod config;
pub mod object;
use self::config::GitConfig;
use self::object::{GitObject, ObjType, Serializable};
use crate::files::is_dir_empty;
use crate::{error::GitError, files};
use configparser::ini::Ini;
use flate2::read::ZlibDecoder;

use byteorder::{BigEndian, ByteOrder};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Sha1;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct GitRepository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub config: GitConfig,
}

#[derive(Debug)]
pub struct GitIndex {
    pub ctime_s: u32,
    pub ctime_n: u32,
    pub mtime_s: u32,
    pub mtime_n: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub sha1: String,
    pub flags: u16,
    pub path: String,
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
                return Some(path);
            } else {
                return None;
            }
        } else if mkdir {
            create_dir_all(&path).unwrap();
            return Some(path);
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
        let gitdir = path.join(&path!(".git"));
        let conf = Ini::new();

        GitRepository {
            worktree: path.to_path_buf(),
            gitdir,
            config: GitConfig::new(conf),
        }
    }

    // Write a new git repository to the path.

    pub fn write_to_path(path: &Path) -> Result<GitRepository, GitError> {
        let repo = GitRepository::new(path);
        if path != path!(".") {
            GitRepository::create_repo_dir(&repo, path)?;
        }
        repo.repo_dir(&path!("branches"), true);
        repo.repo_dir(&path!("objects"), true);
        repo.repo_dir(&path!("refs/tags"), true);
        repo.repo_dir(&path!("refs/heads"), true);
        GitRepository::create_repo_file(
            &repo,
            &path!("description"),
            "Unnamed repository; edit this file 'description' to name the repository.",
        )?;
        GitRepository::create_repo_file(&repo, &path!("HEAD"), "ref: refs/heads/master")?;
        let default_config: GitConfig = Default::default();
        let config = repo.repo_file(&path!("config"))?;
        default_config.save(&config).unwrap();

        Ok(repo)
    }

    // This function creates the repository 's base working directory.
    // It will throw an error if the directory already exists and it is not empty, or if the given path is not a directory.
    pub fn create_repo_dir(repo: &GitRepository, path: &Path) -> Result<(), GitError> {
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

    // Loads an existing git repository
    pub fn load(path: &Path) -> Result<GitRepository, GitError> {
        let gitdir = path.join(path!(".git"));
        let conf_path = gitdir.join(&path!("config"));
        let mut conf = Ini::new();
        conf.load(&conf_path).map_err(|e| {
            GitError::GenericError(format!(
                "Unable to load git config for repo: {}, {}",
                conf_path.to_str().unwrap(),
                e
            ))
        })?;
        Ok(GitRepository {
            worktree: path.to_path_buf(),
            gitdir,
            config: GitConfig::new(conf),
        })
    }

    // This function tries to find a git repository from the current working directory.
    pub fn find() -> Result<GitRepository, GitError> {
        let cwd = files::cwd()?;
        let mut some = Some(cwd);

        while some.is_some() {
            let current = some.as_ref().unwrap();
            let path = current.join(path!(".git"));

            if path.exists() {
                return GitRepository::load(current);
            }
            some = some.unwrap().parent().map(|p| p.to_path_buf());
        }
        Err(GitError::GenericError(
            "Git repository could not be found.".to_owned(),
        ))
    }

    pub fn find_object(&self, name: &str, _format: &ObjType) -> String {
        name.to_owned()
    }

    pub fn read_object(&self, sha: &str) -> Result<GitObject, GitError> {
        let dir = &sha[0..2];
        let rest = &sha[2..];

        let object = self.repo_file(&path!("objects", dir, rest))?;

        let file = File::open(&object).map_err(|_| {
            GitError::PathError("Could not open file".to_owned(), object.to_path_buf())
        })?;

        let mut buf = Vec::new();
        ZlibDecoder::new(file).read_to_end(&mut buf).map_err(|e| {
            GitError::GenericError(format!("Could not read object data: {}", e.to_string()))
        })?;
        let space = buf.iter().position(|b| b == &b' ').unwrap();
        let null = buf.iter().position(|b| b == &b'\x00').unwrap();
        let bytes: &[u8] = buf.as_ref();

        let format = &bytes[0..space];
        let data = &bytes[null + 1..];

        let size: usize = String::from_utf8(bytes[space..null].to_vec())
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        if size != buf.len() - null - 1 {
            return Err(GitError::ObjectError(format!(
                "Invalid object {0}: bad length",
                sha
            )));
        }
        let object_type = ObjType::deserialize(format);
        Ok(GitObject::new(object_type, data))
    }

    pub fn write_object(repo: &GitRepository, obj: &GitObject) -> Result<String, GitError> {
        let mut data = obj.serialize().to_vec();
        let mut result = Vec::new();
        result.append(&mut obj.obj_type.serialize().to_vec());
        result.push(b' ');
        result.append(&mut obj.obj_type.serialize().to_vec());
        result.push(0_u8);
        result.append(&mut data);

        let sha = Sha1::from(&result).hexdigest();
        let path = repo.repo_file(&path!("objects", &sha[0..2], &sha[2..]))?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(1));

        encoder
            .write_all(&result)
            .and(encoder.finish())
            .and_then(|compressed| {
                let file = File::create(path)?;
                Ok((compressed, file))
            })
            .and_then(|(compressed, mut file)| file.write_all(&compressed))
            .and(Ok(sha.to_owned()))
            .map_err(|e| {
                GitError::GenericError(format!(
                    "Unable to compress and save object data: {} - {}",
                    sha,
                    e.to_string()
                ))
            })
    }
    // It parses commit puts it into an ordered hash map.
    pub fn commit_parse(raw: &[u8]) -> BTreeMap<String, Vec<u8>> {
        let buf = raw.to_vec();
        let mut result = BTreeMap::new();

        let mut current: usize = 0;

        while current < buf.len() {
            let space = buf.iter().position(|b| b == &b' ');
            let nl = buf.iter().position(|b| b == &b'\n');
            if space.is_none() || (nl.is_some() && space.unwrap() < nl.unwrap()) {
                result.insert("data".to_owned(), buf[current + 1..].to_vec());
                return result;
            }

            let space_pos = space.unwrap();
            let key = String::from_utf8(buf[current..space_pos].to_vec()).unwrap();

            // Find the end of the value.  Continuation lines begin with a
            // space, so we loop until we find a "\n" not followed by a space.

            let mut it = space_pos + 1;
            let end_pos = loop {
                let newline = buf[it..].iter().position(|b| b == &b'\n');
                if newline.is_none() {
                    break buf.len();
                } else {
                    let newline_pos = newline.unwrap();
                    if buf[newline_pos + 1] != b' ' {
                        break newline_pos;
                    }
                    it = newline_pos + 1;
                }
            };
            let mut value =
                GitRepository::remove_spaces_after_newline(&buf[space_pos + 1..end_pos]);

            // Don't overwrite existing value, but append to it
            if result.contains_key(&key) {
                let mut previous = result.get(&key).unwrap().to_vec();
                previous.append(&mut value);
                result.insert(key, previous);
            } else {
                result.insert(key, value);
            }

            current = end_pos + 1;
        }

        result
    }
    fn remove_spaces_after_newline(input: &[u8]) -> Vec<u8> {
        if input.len() <= 1 {
            return Vec::from(input);
        }

        let mut result = Vec::new();
        let mut idx = 1;
        while idx < input.len() + 1 {
            result.push(input[idx - 1]);
            if idx < input.len() && input[idx - 1] == b'\n' && input[idx] == b' ' {
                idx += 2;
            } else {
                idx += 1;
            }
        }

        result
    }

    pub fn read_index() -> Result<Vec<GitIndex>, GitError> {
        let data = files::read_data(&path!(".git", "index"));

        if data.is_err() {
            return Err(GitError::PathError(
                String::from("reading file"),
                path!(".git", "index"),
            ));
        }
        let data = data.unwrap();
        let sha = Sha1::from(&data[..data.len() - 20]).digest().bytes();

        if sha != data[data.len() - 20..] {
            return Err(GitError::GenericError("Invalid index checksum".to_owned()));
        }
        let signature = std::str::from_utf8(&data[0..4]).unwrap();

        if signature != "DIRC" {
            return Err(GitError::GenericError(format!(
                "Invalid index signature {}",
                signature
            )));
        }

        let version: u32 = u32::from_be_bytes(data[4..8].try_into().expect("Incorrect length"));

        let n_entries: u32 = u32::from_be_bytes(data[8..12].try_into().expect("Invalid index file"));
        if version != 2 {
            return Err(GitError::GenericError(String::from(
                "Unsupported version number.",
            )));
        }
        let entry_data = data[12..data.len() - 20].to_vec();
        let mut i = 0;
        let mut entries: Vec<GitIndex> = Vec::new();
        for _k in 0..n_entries {
            let fields_end = i + 62;
            let ctime_s = BigEndian::read_u32(&entry_data[i..i + 4]);
            let ctime_n = BigEndian::read_u32(&entry_data[i + 4..i + 8]);
            let mtime_s = BigEndian::read_u32(&entry_data[i + 8..i + 12]);
            let mtime_n = BigEndian::read_u32(&entry_data[i + 12..i + 16]);
            let dev = BigEndian::read_u32(&entry_data[i + 16..i + 20]);
            let ino = BigEndian::read_u32(&entry_data[i + 20..i + 24]);
            let mode = BigEndian::read_u32(&entry_data[i + 24..i + 28]);
            let uid = BigEndian::read_u32(&entry_data[i + 28..i + 32]);
            let gid = BigEndian::read_u32(&entry_data[i + 32..i + 36]);
            let size = BigEndian::read_u32(&entry_data[i + 36..i + 40]);
            let s_ha1: String = format!("{:x?}", &entry_data[i + 40..i + 60])
                .split(", ")
                .collect();
            let sha1 = s_ha1[1..s_ha1.len() - 1].to_owned();
            let flags = BigEndian::read_u16(&entry_data[i + 60..i + 62]);
            let mut path_end = fields_end;
            while entry_data[path_end] != 0 {
                path_end += 1;
            }
            // let path_end = memchr(b'0', &entry_data[fields_end..]).unwrap();
            let path = std::str::from_utf8(&entry_data[fields_end..path_end]).unwrap();
            entries.push(GitIndex {
                ctime_s,
                ctime_n,
                mtime_s,
                mtime_n,
                dev,
                ino,
                mode,
                uid,
                gid,
                size,
                sha1,
                flags,
                path: path.to_string(),
            });
            let entry_len: usize = ((62 + path.len() + 8) / 8) as usize * 8;
            i += entry_len;
        }
        Ok(entries)
    }
}
