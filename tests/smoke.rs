extern crate env_logger;
extern crate futures;
extern crate mio_uds;
extern crate tokio_core;
extern crate tokio_uds;

use std::fs;
use std::os::unix::net;
use std::sync::mpsc::channel;
use std::thread;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use tokio_uds::{UnixListener, UnixStream};

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

#[test]
fn connect() {
    let sock_name = "/tmp/tokio-uds-test.socket";

    drop(env_logger::init());
    drop(fs::remove_file(sock_name));

    let l = t!(Core::new());
    let srv = t!(net::UnixListener::bind(sock_name));
    let t = thread::spawn(move || t!(srv.accept()).0);

    let stream = UnixStream::connect(sock_name, &l.handle());

    let mine = t!(stream);
    let theirs = t.join().unwrap();

    println!("mine: {:?}", mine);
    println!("theirs: {:?}", theirs);

    assert_eq!(t!(mine.peer_addr()).is_unnamed(), false);
    assert_eq!(t!(theirs.local_addr()).is_unnamed(), false);

    assert_eq!(
        t!(mine.local_addr()).as_pathname(),
        t!(theirs.peer_addr()).as_pathname()
    );
    assert_eq!(
        t!(theirs.local_addr()).as_pathname(),
        t!(mine.peer_addr()).as_pathname()
    );

    drop(fs::remove_file(sock_name));
}

#[test]
fn accept() {
    let sock_name = "/tmp/tokio-uds-test-2.socket";

    drop(env_logger::init());
    drop(fs::remove_file(sock_name));

    let mut l = t!(Core::new());
    let srv = t!(UnixListener::bind(sock_name, &l.handle()));

    let (tx, rx) = channel();
    let client = srv.incoming()
        .map(move |t| {
            tx.send(()).unwrap();
            t.0
        })
        .into_future()
        .map_err(|e| e.0);
    assert!(rx.try_recv().is_err());
    let t = thread::spawn(move || net::UnixStream::connect(sock_name).unwrap());

    let (mine, _remaining) = t!(l.run(client));
    let mine = mine.unwrap();
    let theirs = t.join().unwrap();

    assert_eq!(
        t!(mine.local_addr()).as_pathname(),
        t!(theirs.peer_addr()).as_pathname()
    );
    assert_eq!(
        t!(theirs.local_addr()).as_pathname(),
        t!(mine.peer_addr()).as_pathname()
    );

    drop(fs::remove_file(sock_name));
}

#[test]
fn accept2() {
    let sock_name = "/tmp/tokio-uds-test-3.socket";

    drop(env_logger::init());
    drop(fs::remove_file(sock_name));

    let mut l = t!(Core::new());
    let srv = t!(UnixListener::bind(sock_name, &l.handle()));

    let t = thread::spawn(move || net::UnixStream::connect(sock_name).unwrap());

    let (tx, rx) = channel();
    let client = srv.incoming()
        .map(move |t| {
            tx.send(()).unwrap();
            t.0
        })
        .into_future()
        .map_err(|e| e.0);
    assert!(rx.try_recv().is_err());

    let (mine, _remaining) = t!(l.run(client));
    mine.unwrap();
    t.join().unwrap();

    drop(fs::remove_file(sock_name));
}
