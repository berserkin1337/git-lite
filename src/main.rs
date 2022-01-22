#[macro_use]
extern crate clap;
use crate::error::GitError;
use crate::repository::GitRepository;
use clap::{App, ArgMatches};
use repository::object::{GitObject, ObjType, Serializable};
use std::path::Path;

#[macro_use]
pub mod macros;
pub mod error;
pub mod files;
pub mod repository;
pub mod test;
fn main() {
    let yml = load_yaml!("args.yaml");
    let app = App::from_yaml(yml).get_matches();
    if let Some(app) = app.subcommand_matches("init") {
        init(app);
    } else if let Some(matches) = app.subcommand_matches("cat-file") {
        cat_file(matches).unwrap();
    } else if let Some(matches) = app.subcommand_matches("hash-object") {
        hash_object(matches).unwrap();
    } else if let Some(_matches) = app.subcommand_matches("ls-files") {
        ls_files().unwrap();
    } else if let Some(matches) = app.subcommand_matches("commit") {
        commit(matches);
    } }

fn init(matches: &ArgMatches) {
    if matches.is_present("path") {
        let repo_path = matches.value_of("path").unwrap();
        let result = GitRepository::write_to_path(Path::new(repo_path));
        if result.is_err() {
            println!("{}", result.unwrap_err());
        }
    } else {
        let repo_path = ".";
        let result = GitRepository::write_to_path(Path::new(repo_path));
        if result.is_err() {
            println!("{}", result.unwrap_err());
        }
    }
}

fn cat_file(matches: &ArgMatches) -> Result<(), GitError> {
    let object = matches.value_of("object").unwrap();
    let object_type: ObjType = ObjType::deserialize(matches.value_of("type").unwrap().as_bytes());
    GitRepository::load(&path!("."))
        .and_then(|repo| {
            let object = repo.find_object(object, &object_type);
            repo.read_object(&object)
        })
        .and_then(|obj| {
            String::from_utf8(obj.serialize().to_vec()).map_err(|e| {
                GitError::GenericError(format!(
                    "Error converting object data to string: {}",
                    e.to_string()
                ))
            })
        })
        .map(|object_as_string| {
            println!("{}", object_as_string);
        })
}

fn hash_object(matches: &ArgMatches) -> Result<(), GitError> {
    let mut repo = None;

    if matches.is_present("write") {
        GitRepository::find().map(|found| {
            repo = Some(found);
        })?;
    }
    let objtype = ObjType::deserialize(matches.value_of("type").unwrap().as_bytes());
    let path = path!(matches.value_of("path").unwrap());
    let data = files::read_data(&path)?;
    let object = GitObject::new(objtype, &data);

    if let Some(repo) = repo {
        GitRepository::write_object(&repo, &object).map(|sha| {
            println!("{}", sha);
        })?;
    }
    Ok(())
}
fn ls_files() -> Result<(), GitError> {
    let entries = GitRepository::read_index().unwrap();
    for entry in entries {
        println!("{}", entry.path);
    }
    Ok(())
}

fn commit(matches: &ArgMatches) {
    GitRepository::commit(
        matches.value_of("message").unwrap().to_string(),
        matches.value_of("author").unwrap().to_string(),
    );
}
