use std::{env, thread};

use dotenv::dotenv;
use futures::{future, Future, Stream};
use futures::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::sync::oneshot::{self, Sender};
use mercury::{Article, Mercury};
use tokio_core::reactor::Core;

use error::{Error, Result};

pub type Message = (Sender<Result<Article>>, String);

#[derive(Debug)]
pub struct Reader {
    tx: UnboundedSender<Message>,
}

impl Reader {
    pub fn new() -> Result<Reader> {
        let (tx, rx) = mpsc::unbounded();
        let (cb, f) = oneshot::channel();

        thread::spawn(|| worker(cb, rx));

        f.wait()?;
        Ok(Reader { tx })
    }

    pub fn parse(&self, url: &str) -> Result<Article> {
        let (cb, resp) = oneshot::channel();
        let msg = (cb, url.to_owned());

        if let Err(_) = self.tx.unbounded_send(msg) {
            bail!("receiver was dropped");
        }

        resp.wait()?
    }
}

fn worker(cb: Sender<()>, rx: UnboundedReceiver<Message>) -> Result<()> {
    dotenv().ok();

    let mut core = Core::new()?;
    let handle = core.handle();

    let key = env::var("MERCURY_API_KEY")?;
    let merc = Mercury::new(&handle, key)?;

    let queue = rx.for_each(move |(cb, url)| {
        merc.parse(&url)
            .map_err(Error::from)
            .then(|result| cb.send(result).map_err(|_| ()))
    });

    if let Err(_) = cb.send(()) {
        bail!("receiver was dropped");
    }

    handle.spawn(queue);
    Ok(core.run(future::empty::<_, Error>())?)
}
