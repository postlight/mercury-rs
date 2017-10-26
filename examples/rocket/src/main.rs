#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate dotenv;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate mercury;
extern crate rocket;
extern crate rocket_contrib;
extern crate tokio_core;

mod error;
mod reader;

use std::path::{Path, PathBuf};

use rocket_contrib::Template;
use rocket::response::NamedFile;
use rocket::State;

use error::Result;
use reader::Reader;

quick_main!(run);

#[get("/read?<url>")]
fn read(url: &str, reader: State<Reader>) -> Result<Template> {
    let article = reader.parse(&url[4..])?;
    Ok(Template::render("index", &article))
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn run() -> Result<()> {
    let e = rocket::ignite()
        .mount("/", routes![files, read])
        .attach(Template::fairing())
        .manage(Reader::new()?)
        .launch();

    Err(e.into())
}
