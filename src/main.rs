mod config;

const APP_NAME: &str = "Natten Multi Repo Tool";
const APP_VERSION: &str = "0.0.1";

fn main() {
    clap::App::new(APP_NAME)
        .version(APP_VERSION)
        .get_matches();


    config::loader::load_config("Heisann du");


    // println!("Starting {} version {}", APP_NAME, APP_VERSION);


}
