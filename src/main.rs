#[macro_use]
extern crate clap;


use clap::{App, ArgMatches};
fn main() {
    let yml = load_yaml!("args.yaml");
    let app = App::from_yaml(yml).get_matches();
    
    if let Some(app) =  app.subcommand_matches("init"){
        println!("Thou have chosen init.");
    }
}
