use crate::error::GitError;
use crate::repository::GitRepository;
use clap::{arg, App};
use clap::{AppSettings, Arg, ArgMatches};

use repository::object::{GitObject, ObjType, Serializable};
use std::path::{Path};

#[macro_use]
pub mod macros;
pub mod error;
pub mod files;
pub mod repository;
pub mod test;
fn main() {
    // let yml = load_yaml!("args.yaml");
    // let app = App::from_yaml(yml).get_matches();
    // if let Some(matches) = app.subcommand_matches("init") {
    //     init(matches);
    // } else if let Some(matches) = app.subcommand_matches("cat-file") {
    //     cat_file(matches).unwrap();
    // } else if let Some(matches) = app.subcommand_matches("hash-object") {
    //     hash_object(matches).unwrap();
    // } else if let Some(_matches) = app.subcommand_matches("ls-files") {
    //     ls_files().unwrap();
    // } else if let Some(matches) = app.subcommand_matches("commit") {
    //     commit(matches);
    // } else {
    //     App::from_yaml(yml).print_help().unwrap();
    // }
    let mut matches = App::new(env!("CARGO_CRATE_NAME"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .about("implementation of git in rust.")
        .subcommand(
            App::new("init")
                .about("Creates a new git repository or reinitializes an existing one.")
                .arg(
                    Arg::new("path")
                        .takes_value(true)
                        .short('p')
                        .value_name("path")
                        .help("specify the repository's path"),
                ),
        )
        .subcommand(
            App::new("cat-file")
                .about("Provide content or type and size information for repository objects")
                .arg(
                    Arg::new("type")
                        .takes_value(true)
                        .short('t')
                        .value_name("type")
                        .help("specify the type of the object"),
                )
                .arg(
                    Arg::new("object")
                        .takes_value(true)
                        .short('o')
                        .value_name("object")
                        .help("The name of the object to show"),
                ),
        )
        .subcommand(
            App::new("hash-object")
                .about("Compute object ID and optionally creates a blob from a file")
                .arg(
                    Arg::new("type")
                        .takes_value(true)
                        .value_name("type")
                        .short('t')
                        .default_value("blob")
                        .help("specify the type of the object"),
                )
                .arg(
                    Arg::new("write")
                        .takes_value(false)
                        .short('w')
                        .help("Actually write the object into the database"),
                )
                .arg(
                    Arg::new("path")
                        .value_name("path")
                        .short('p')
                        .takes_value(true)
                        .help("The path of the object"),
                ),
        )
        .subcommand(App::new("ls-files").about("Lists the files in the git index"))
        .subcommand(
            App::new("commit")
                .about("Record changes to the repository")
                .arg(
                    Arg::new("message")
                        .short('m')
                        .takes_value(true)
                        .value_name("message")
                        .help("use the given message as the commit message"),
                )
                .arg(
                    Arg::new("author")
                        .short('a')
                        .takes_value(true)
                        .value_name("author")
                        .help("use the given author as the author of the commit"),
                ),
        )
        .subcommand(
            App::new("add")
                .about("Add file contents to the index")
                .arg(arg!(<path> ... "Stuff to add").allow_invalid_utf8(true).short('p')),
        );
    let get = matches.get_matches_mut();

    match get.subcommand() {
        Some(("init", sub_matches)) => {
            init(sub_matches);
        }
        Some(("cat-file", sub_matches)) => {
            cat_file(sub_matches).unwrap();
        }
        Some(("hash-object", sub_matches)) => {
            hash_object(sub_matches).unwrap();
        }
        Some(("ls-files", _sub_matches)) => {
            ls_files().unwrap();
        }
        Some(("commit", sub_matches)) => {
            commit(sub_matches);
        }
        Some(("add", sub_matches)) => {
            git_add(sub_matches);
        }
        _ => {
            matches.print_help().unwrap();
        }
    }
}

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
                GitError::GenericError(format!("Error converting object data to string: {}", e))
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

fn git_add(sub_matches: &ArgMatches) {
    let paths: Vec<String> = sub_matches
        .value_of("path")
        .unwrap()
        .to_string()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    
    GitRepository::add_git(&paths)
}
