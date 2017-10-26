use hyper::error::{Error as HyperError, UriError};
use native_tls::Error as TlsError;
use serde_json::Error as JsonError;

error_chain!{
    foreign_links {
        Http(HyperError);
        Json(JsonError);
        Tls(TlsError);
    }
}

impl From<UriError> for Error {
    fn from(e: UriError) -> Error {
        HyperError::from(e).into()
    }
}
