//! The official Rust client for the [Mercury Parser].
//!
//! With just one API request, Mercury takes any web article and returns only the
//! relevant content — headline, author, body text, relevant images and more — free
//! from any clutter. It’s reliable, easy-to-use and free.
//!
//! * [Homepage]
//! * [Source Code]
//!
//! ## Usage
//!
//! The examples in this document assume you already have a Mercury Parser API key. If
//! you do not already have one, you can [sign up here][Mercury Parser].
//!
//! ### Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! futures = "0.1"
//! mercury = "0.1"
//! tokio-core = "0.1"
//! ```
//!
//! ### Example
//!
//! Additional examples can be found on [GitHub][Source Code].
//!
//! ```
//! # extern crate dotenv;
//! # extern crate futures;
//! # extern crate mercury;
//! # extern crate tokio_core;
//! #
//! # use std::env;
//! # use std::error::Error;
//! #
//! # use dotenv::dotenv;
//! # use futures::Future;
//! # use mercury::Mercury;
//! # use tokio_core::reactor::Core;
//! #
//! # fn run() -> Result<(), Box<Error>> {
//! // Create a new event loop with tokio.
//! let mut core = Core::new()?;
//!
//! // Load your API key from the environment.
//! let key = env::var("MERCURY_API_KEY")?;
//!
//! // Pass a handle to the event loop and the API key to the Mercury constructor.
//! let client = Mercury::new(&core.handle(), key)?;
//!
//! // The parse method returns a Future that will resolve to a parsed Article.
//! let resp = client.parse("https://example.com").inspect(|article| {
//!     println!("{:#?}", article);
//! });
//!
//! // Block the current thread until the future completes.
//! core.run(resp)?;
//! #
//! # Ok(())
//! # }
//! #
//! #
//! # fn main() {
//! # dotenv().ok();
//! # run().unwrap();
//! # }
//! ```
//!
//! [Homepage]: https://mercury.postlight.com
//! [Mercury Parser]: https://mercury.postlight.com/web-parser
//! [Source Code]: https://github.com/postlight/mercury-rs

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
#[macro_use]
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate num_cpus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

mod article;

/// Types representing errors that can occur.
pub mod error;

use std::rc::Rc;

use futures::{stream, Future, Poll, Stream};
use futures::future::{self, FutureResult};
use hyper_tls::HttpsConnector;
use hyper::{Get, Request, Uri};
use hyper::client::{Client, HttpConnector};
use tokio_core::reactor::Handle;

pub use article::*;
pub use error::Error;

type Connect = HttpsConnector<HttpConnector>;

/// A client used to make requests to the Mercury Parser API.
///
/// See the [module-level documentation] for more details.
/// [module-level documentation]: ./index.html
#[derive(Debug)]
pub struct Mercury(Rc<Inner>);

impl Mercury {
    /// Create a new Mercury client.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate dotenv;
    /// # extern crate mercury;
    /// # extern crate tokio_core;
    /// #
    /// # use std::env;
    /// # use std::error::Error;
    /// #
    /// # use dotenv::dotenv;
    /// # use mercury::Mercury;
    /// # use tokio_core::reactor::Core;
    /// #
    /// # fn main() {
    /// #     dotenv().ok();
    /// #     example().unwrap();
    /// # }
    /// #
    /// # fn example() -> Result<(), Box<Error>> {
    /// let core = Core::new()?;
    /// let key = env::var("MERCURY_API_KEY")?;
    ///
    /// Mercury::new(&core.handle(), key)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(handle: &Handle, key: String) -> Result<Mercury, Error> {
        Inner::new(handle, key).map(Rc::new).map(Mercury)
    }

    /// Send a request to the Mercury Parser API using this client.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate dotenv;
    /// # extern crate futures;
    /// # extern crate mercury;
    /// # extern crate tokio_core;
    /// #
    /// # use std::env;
    /// # use std::error::Error;
    /// #
    /// # use dotenv::dotenv;
    /// # use futures::Future;
    /// # use mercury::Mercury;
    /// # use tokio_core::reactor::Core;
    /// #
    /// # fn main() {
    /// #     dotenv().ok();
    /// #     example().unwrap();
    /// # }
    /// #
    /// # fn example() -> Result<(), Box<Error>> {
    /// # let mut core = Core::new()?;
    /// # let client = Mercury::new(&core.handle(), env::var("MERCURY_API_KEY")?)?;
    /// #
    /// let resp = client.parse("https://example.com").inspect(|article| {
    ///     println!("{}", article.content);
    /// });
    /// # core.run(resp)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(&self, resource: &str) -> Response {
        let mrcy = Mercury::clone(self);
        let f = build_url(resource).and_then(move |url| {
            let mut req = Request::new(Get, url);

            header!{ (XApiKey, "X-Api-Key") => [String] }
            req.headers_mut().set(XApiKey(mrcy.key()));

            mrcy.client()
                .request(req)
                .and_then(|resp| resp.body().map(stream::iter_ok).flatten().collect())
                .map_err(Error::from)
                .and_then(|body| match serde_json::from_slice(&body)? {
                    ParserResult::Ok(article) => Ok(article),
                    ParserResult::Err { msg, msgs } => bail!(msg.unwrap_or(msgs)),
                })
        });

        Response::new(Box::new(f))
    }

    /// Returns a reference to the underlying hyper client.
    fn client(&self) -> &Client<Connect> {
        let Mercury(ref inner) = *self;
        &inner.client
    }

    /// Returns an owned copy of the API key.
    fn key(&self) -> String {
        let Mercury(ref inner) = *self;
        inner.key.to_owned()
    }
}

impl Clone for Mercury {
    fn clone(&self) -> Mercury {
        let Mercury(ref inner) = *self;
        Mercury(Rc::clone(inner))
    }
}

/// A [`Future`] that will resolve to a parsed [`Article`].
///
/// [`Article`]: ./struct.Article.html
/// [`Future`]: ../futures/future/trait.Future.html
#[must_use = "futures do nothing unless polled"]
pub struct Response(Box<Future<Item = Article, Error = Error>>);

impl Response {
    fn new(f: Box<Future<Item = Article, Error = Error>>) -> Response {
        Response(f)
    }
}

impl Future for Response {
    type Item = Article;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let Response(ref mut f) = *self;
        f.poll()
    }
}

#[derive(Debug)]
struct Inner {
    client: Client<Connect>,
    key: String,
}

impl Inner {
    fn new(handle: &Handle, key: String) -> Result<Inner, Error> {
        let conn = Connect::new(num_cpus::get(), handle)?;
        let client = Client::configure().connector(conn).build(handle);

        Ok(Inner { client, key })
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
#[derive(Deserialize)]
#[serde(untagged)]
enum ParserResult {
    Ok(Article),
    Err {
        #[serde(rename = "message")] msg: Option<String>,
        #[serde(default, rename = "messages")] msgs: String,
    },
}

fn build_url(resource: &str) -> FutureResult<Uri, Error> {
    const ENDPOINT: &'static str = "https://mercury.postlight.com/parser";

    let mut raw = String::with_capacity(ENDPOINT.len() + resource.len() + 5);

    raw.push_str(ENDPOINT);
    raw.push_str("?url=");
    raw.push_str(resource);

    match raw.parse() {
        Ok(uri) => future::ok(uri),
        Err(e) => future::err(e.into()),
    }
}
