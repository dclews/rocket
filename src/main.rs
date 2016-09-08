// extern crate nasm;
extern crate git2;
extern crate regex;

pub mod payload;
pub mod app;

fn main() {
    let mut application = match app::App::new() {
        Ok(a) => a,
        Err(e) => {
            println!("Error: {}", e);
            panic!(app::App::usage());
        },
    };
    application.run();
}
