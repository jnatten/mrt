#[macro_use]
extern crate serde_derive;

mod config;

const APP_NAME: &str = "Multi Repo Tool";
const APP_VERSION: &str = "0.0.1";

fn main() {
    clap::App::new(APP_NAME)
        .version(APP_VERSION)
        .get_matches();


    let config = config::loader::load_config("config.json");

    match config {
        Ok(_) => println!("OK"),
        _ => println!("NO")
    }


}
