error_chain!{
    links {
        Mercury(::mercury::error::Error, ::mercury::error::ErrorKind);
    }

    foreign_links {
        Env(::std::env::VarError);
        Io(::std::io::Error);
    }
}
