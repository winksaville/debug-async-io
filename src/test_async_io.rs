//use std::future::Future;
use std::io;
//use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::net::{TcpListener, TcpStream};
#[cfg(unix)]
//use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};
//use std::sync::Arc;
use std::thread;
use std::time::Duration;

//use async_io::{Async, Timer};
use async_io::{Async};
//use futures_lite::{future, prelude::*};
use futures_lite::{future};

use crate::env_logger_init;

//use simple_executor::{executor::*, timer::*};
use simple_executor::executor::*;

pub(crate) fn not_missing_wake() -> io::Result<()> {
    env_logger_init();
    log::trace!("not_missing_wake:+; block_on+");

    fn abc() -> &'static str {
        "abc"
    }

    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        log::trace!("not_missing_wake:+; block_on TOP; bind+ {}", abc());

        let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 0)).expect("Unexpected bind failure");
        log::trace!("not_missing_wake: bind-; get_ref+");
        let addr = listener.get_ref().local_addr().expect("Unexpected local_addr failure");
        log::trace!("not_missing_wake: get_ref-");


        log::trace!("not_missing_wake: sleep+ 1000ms");
        let delay = Duration::from_millis(1000);
        thread::sleep(delay);
        log::trace!("not_missing_wake: sleep- 1000ms; spawn accept");

        //My spawner.spawn doesn't return a future, so the
        //`let stream1 = task.await?.0;` doesn't work :(

        let task = spawner.spawn(async move {
            log::trace!("not_missing_wake: accept.await+");
            let res = listener.accept().await;
            log::trace!("not_missing_wake: accept.await- {:?}", res);

            res
        });
        log::trace!("not_missing_wake: spawn- accept; connect+");

        let stream2 = Async::<TcpStream>::connect(addr).await?;
        log::trace!("not_missing_wake: connect-");

        let stream1 = task.await?.0;
        log::trace!("not_missing_wake: task.wait-; test stream1.peer_addr == stream2.local_addr");

        //assert_eq!(
        //    stream1.get_ref().peer_addr()?,
        //    stream2.get_ref().local_addr()?,
        //);

        //log::trace!("not_missing_wake: test stream2.peer_addr == stream1.local_addr");
        //assert_eq!(
        //    stream2.get_ref().peer_addr()?,
        //    stream1.get_ref().local_addr()?,
        //);

        log::trace!("not_missing_wake: block_on BTM");
    });

    // Drop the spawner so that our executor knows it is finished and won't
    // receive more incoming tasks to run.
    drop(spawner);

    let now = std::time::SystemTime::now();

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    executor.run();

    match now.elapsed() {
        Ok(elapsed) => assert!(elapsed >= Duration::from_millis(DELAY_MS)),
        Err(e) => panic!("{}", e),
    }

    log::trace!("not_missing_wake:-");
    res
}
