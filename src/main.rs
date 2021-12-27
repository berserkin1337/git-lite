#[macro_use]
extern crate clap;
use std::path::Path;

use crate::repository::GitRepository;
use crate::error::GitError;
use clap::{App, ArgMatches};




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