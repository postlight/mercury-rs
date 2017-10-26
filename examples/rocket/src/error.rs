use std::env::VarError;
use std::io::Error as IoError;

use futures::sync::oneshot::Canceled;
use mercury::Error as MercuryError;
use rocket::error::LaunchError;

error_chain!{
    links {
        Mercury(MercuryError, ::mercury::error::ErrorKind);
    }

    foreign_links {
        Canceled(Canceled);
        Env(VarError);
        Io(IoError);
        Launch(LaunchError);
    }
}
