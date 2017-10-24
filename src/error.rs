use hyper::error::UriError;

error_chain!{
    foreign_links {
        Http(::hyper::error::Error);
        Json(::serde_json::Error);
        Tls(::native_tls::Error);
    }
}

impl From<UriError> for Error {
    fn from(e: UriError) -> Error {
        ErrorKind::Http(e.into()).into()
    }
}
