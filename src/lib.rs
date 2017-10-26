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
//! # use std::{env, error};
//! #
//! # use dotenv::dotenv;
//! # use futures::Future;
//! # use mercury::Mercury;
//! # use tokio_core::reactor::Core;
//! #
//! # type Error = Box<error::Error>;
//! #
//! # fn main() {
//! #     dotenv().ok();
//! #     example().unwrap();
//! # }
//! #
//! # fn example() -> Result<(), Error> {
//! // Create a new event loop with tokio.
//! let mut core = Core::new()?;
//! let handle = core.handle();
//!
//! // Load your API key from the environment.
//! let key = env::var("MERCURY_API_KEY")?;
//!
//! // Pass a handle to the event loop and the API key to the Mercury constructor.
//! let client = Mercury::new(&handle, key)?;
//!
//! // The parse method returns a Future that will resolve to a parsed Article.
//! let future = client.parse("https://example.com").inspect(|article| {
//!     println!("{:#?}", article);
//! });
//!
//! // Block the current thread until the future completes.
//! core.run(future)?;
//! #
//! # Ok(())
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

use futures::{stream, Future, IntoFuture, Poll, Stream};
use hyper_tls::HttpsConnector;
use hyper::{Get, Request, Uri};
use hyper::client::{Client, HttpConnector};
use tokio_core::reactor::Handle;

pub use article::*;
pub use error::Error;

const ENDPOINT: &'static str = "https://mercury.postlight.com/parser";

type Connect = HttpsConnector<HttpConnector>;

/// A client used to make requests to the [Mercury Parser].
///
/// [Mercury Parser]: https://mercury.postlight.com/web-parser
#[derive(Debug)]
pub struct Mercury(Rc<Inner>);

impl Mercury {
    /// Create a new Mercury client.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate dotenv;
    /// # extern crate futures;
    /// # extern crate mercury;
    /// # extern crate tokio_core;
    /// #
    /// # use std::{env, error};
    /// #
    /// # use dotenv::dotenv;
    /// # use futures::Future;
    /// # use mercury::Mercury;
    /// # use tokio_core::reactor::Core;
    /// #
    /// # type Error = Box<error::Error>;
    /// #
    /// # fn main() {
    /// #     dotenv().ok();
    /// #     example().unwrap();
    /// # }
    /// #
    /// # fn example() -> Result<(), Error> {
    /// let core = Core::new()?;
    /// let handle = core.handle();
    ///
    /// let key = env::var("MERCURY_API_KEY")?;
    /// let client = Mercury::new(&handle, key)?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(handle: &Handle, key: String) -> Result<Mercury, Error> {
        Inner::new(handle, key).map(Rc::new).map(Mercury)
    }

    /// Return a reference to a handle to the event loop this client is associated with.
    pub fn handle(&self) -> &Handle {
        self.client().handle()
    }

    /// Returns a reference to the API key associated with this client.
    pub fn key(&self) -> &str {
        &self.0.key
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
    /// # use std::{env, error};
    /// #
    /// # use dotenv::dotenv;
    /// # use futures::Future;
    /// # use mercury::Mercury;
    /// # use tokio_core::reactor::Core;
    /// #
    /// # type Error = Box<error::Error>;
    /// #
    /// # fn main() {
    /// #     dotenv().ok();
    /// #     example().unwrap();
    /// # }
    /// #
    /// # fn example() -> Result<(), Error> {
    /// # let mut core = Core::new()?;
    /// # let handle = core.handle();
    /// #
    /// # let key = env::var("MERCURY_API_KEY")?;
    /// # let client = Mercury::new(&handle, key)?;
    /// #
    /// let future = client.parse("https://example.com").inspect(|article| {
    ///     println!("{:#?}", article);
    /// });
    /// #
    /// # core.run(future.then(|_| Ok(())))
    /// # }
    /// ```
    pub fn parse(&self, resource: &str) -> Response {
        let merc = Mercury::clone(self);
        let f = build_url(resource).into_future().and_then(move |url| {
            let mut req = Request::new(Get, url);

            header!{ (XApiKey, "X-Api-Key") => [String] }
            req.headers_mut().set(XApiKey(merc.key().to_owned()));

            merc.client()
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
        &self.0.client
    }
}

impl Clone for Mercury {
    /// Increments the strong reference count of the underlying [`Rc`] pointer.
    ///
    /// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
    fn clone(&self) -> Mercury {
        Mercury(Rc::clone(&self.0))
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
        self.0.poll()
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

fn build_url(resource: &str) -> Result<Uri, Error> {
    let mut raw = String::with_capacity(ENDPOINT.len() + resource.len() + 5);

    raw.push_str(ENDPOINT);
    raw.push_str("?url=");
    raw.push_str(resource);

    Ok(raw.parse()?)
}
