extern crate rocket;
use rocket::app;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut application = match app::App::new(args) {
        Ok(a) => a,
        Err(e) => {
            println!("Error: {}", e);
            panic!(app::App::usage());
        }
    };
    application.run();
}
