#[macro_use]
extern crate clap;
#[macro_use]
extern crate nom;
extern crate byteorder;

use clap::{App, Arg};

mod parser;
mod writer;

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new(format!("reaktor-mapper {}", crate_version!()))
        .about("Quickly generates Reaktor sample maps.")
        .arg(Arg::with_name("version").short("V").long("version").help(
            "Prints version info",
        ))
        .arg(Arg::with_name("root").index(1))
        .arg(Arg::with_name("name").index(2))
}

fn main() {
    //let bytes = include_bytes!("../32.map").to_vec();
    //parser::go(&bytes);

    let matches = app().get_matches();
    if matches.is_present("version") {
        println!("reaktor-mapper {}", crate_version!());
        return;
    }

    let root = match matches.value_of("root") {
        Some(x) => x,
        None => {
            println!("No source directory provided.");
            return;
        }
    };

    let name = matches.value_of("name").unwrap_or("output.map");

    writer::map_folder(root, name);
}
