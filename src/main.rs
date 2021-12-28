#[macro_use]
extern crate clap;
use std::path::Path;

use crate::repository::GitRepository;
use crate::error::GitError;
use clap::{App, ArgMatches};
use repository::object::{ObjType, Serializable};




#[macro_use]
pub mod macros;
pub mod repository;
pub mod error;
pub mod files;
fn main() {
    let yml = load_yaml!("args.yml");
    let app = App::from_yaml(yml).get_matches();
    
    if let Some(app) =  app.subcommand_matches("init"){
        init(&app);
    }else if let Some(matches) = app.subcommand_matches("cat-file") {
        cat_file(matches).unwrap();
    }
}


fn init(matches:&ArgMatches){
    if matches.is_present("path") {
        let repo_path = matches.value_of("path").unwrap();
        let result = GitRepository::write_to_path(Path::new(repo_path));
        if result.is_err() {
            println!("{}", result.unwrap_err());
        }
         
    }else{
        let repo_path = ".";
        let result = GitRepository::write_to_path(Path::new(repo_path));
        if result.is_err() {
            println!("{}", result.unwrap_err());
        }
    }
}


fn cat_file(matches:&ArgMatches)  -> Result<(), GitError>{
    let object = matches.value_of("object").unwrap();
    let object_type: ObjType = ObjType::deserialize(matches.value_of("type").unwrap().as_bytes());
    GitRepository::load(&path!("."))
        .and_then(|repo| {
            let object = repo.find_object(object, &object_type);
            repo.read_object(&object)
        })
        .and_then(|obj| {
            String::from_utf8(obj.serialize().to_vec())
                .map_err(|e| GitError::GenericError(format!("Error converting object data to string: {}", e.to_string())))
        })
        .and_then(|object_as_string| {
            println!("{}", object_as_string);
            Ok(())
        })
}