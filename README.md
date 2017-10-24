# mercury-rs

[![CircleCI branch](https://img.shields.io/circleci/token/2a269417e742b12343eab366752dbb8d837cb1e8/project/github/postlight/mercury-rs/master.svg?style=flat-square)](https://circleci.com/gh/postlight/mercury-rs/tree/master)

The official Rust client for the [Mercury Parser](https://mercury.postlight.com/web-parser).

With just one API request, Mercury takes any web article and returns only the relevant content — headline, author, body text, relevant images and more — free from any clutter. It’s reliable, easy-to-use and free.

* [Documentation](https://docs.rs/mercury)
* [Homepage](https://mercury.postlight.com)
* [Examples](./examples)

## Installation

The examples in this document assume you already have a Mercury Parser API key. If you do not already have one, you can [sign up here](https://mercury.postlight.com/web-parser).

Add this to your `Cargo.toml`:

```toml
[dependencies]
futures = "0.1"
mercury = "0.1"
tokio-core = "0.1"
```

Add this to your `main.rs`:

```rust
extern crate futures;
extern crate mercury;
extern crate tokio_core;
```

## Usage

```rust
// Create a new event loop with tokio.
let mut core = Core::new()?;

// Load your API key from the environment.
let key = env::var("MERCURY_API_KEY")?;

// Pass a handle to the event loop and the API key to the Mercury constructor.
let client = Mercury::new(&core.handle(), key)?;

// The parse method returns a Future that will resolve to a parsed Article.
let resp = client.parse("https://example.com").inspect(|article| {
    println!("{:#?}", article);
});

// Block the current thread until the future completes.
core.run(resp)?;
```

*Additional examples can be found [here](./examples).*

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
