use std::env::VarError;
use std::io::Error as IoError;

use mercury::Error as MercuryError;

error_chain!{
    links {
        Mercury(MercuryError, ::mercury::error::ErrorKind);
    }

    foreign_links {
        Env(VarError);
        Io(IoError);
    }
}
