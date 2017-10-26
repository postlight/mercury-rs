extern crate clap;
extern crate dotenv;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate html2text;
extern crate mercury;
extern crate tokio_core;

mod error;

use std::env;
use std::io::{self, Write};

use clap::{App, AppSettings};
use dotenv::dotenv;
use futures::Future;
use mercury::{Article, Mercury};
use tokio_core::reactor::Core;

use error::{Error, Result};

quick_main!(run);

fn run() -> Result<i32> {
    dotenv().ok();

    let mut core = Core::new()?;
    let handle = core.handle();

    let key = env::var("MERCURY_API_KEY")?;
    let client = Mercury::new(&handle, key)?;

    let matches = App::new("Mercury Reader")
        .version("0.1")
        .about("Read articles in your terminal. Powered by the Mercury Parser.")
        .author("Postlight")
        .arg_from_usage("<url> 'The url of the article you would like to read'")
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let url = matches.value_of("url").unwrap_or_else(|| unreachable!());
    let task = client.parse(url).map_err(Error::from).and_then(render);

    core.run(task)
}

fn render(article: Article) -> Result<i32> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    handle.write(&[10])?;

    if let Some(ref name) = article.author {
        handle.write(name.as_bytes())?;
        handle.write(&[10])?;
    }

    handle.write(article.title.as_bytes())?;
    handle.write(&[10, 10])?;

    handle.write({
        let data = article.content.as_bytes();
        let width = article.content.len();

        html2text::from_read(data, width).as_bytes()
    })?;

    handle.write(&[10])?;

    Ok(0)
}
