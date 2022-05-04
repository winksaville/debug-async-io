# debug-async-io

At the moment, this isn't working as I was trying to
add test_async_io but was unable to integrate with my
simple-executor as spawner.spawn of simple-executor
doesn't return a future. I'm not sure how to resolve
this so stopping developement for now.


I’ve had trouble getting the [hole_punching](https://docs.rs/libp2p/0.44.0/libp2p/tutorials/hole_punching/index.html) working.
in step 3 of the “[Setting up the relay server](https://docs.rs/libp2p/0.44.0/libp2p/tutorials/hole_punching/index.html#setting-up-the-relay-server)”
I run a debug build of the relay server on a Digital Ocean VM with 1CPU, 1GB RAM and 25GB SSD using
`RUST_LOG=trace relay_v2.debug --port 4001 --secret-key-seed 0 &> relay_v2.debug.log`.
And I then run run [`libp2p-lookup direct --address /ip4/$RELAY_SERVER_IP/tcp/4001`](https://github.com/mxinden/libp2p-lookup)
on my desktop computer, 3900x. And the libp2p-lookup almost always fails with a timeout.
```
$ libp2p-lookup direct --address /ip4/164.92.118.108/tcp/4001
[2022-04-19T16:19:35Z ERROR libp2p_lookup] Lookup failed: Timeout.
```

But if I run a release build of the relay server
`RUST_LOG=trace relay_v2.release --port 4001 --secret-key-seed 0 &> relay_v2.debug.log` 
it "always" succeeds.
```
$ libp2p-lookup direct --address /ip4/164.92.118.108/tcp/4001
Lookup for peer with id PeerId("12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN") succeeded.


Protocol version: "/TODO/0.0.1"
Agent version: "rust-libp2p/0.36.0"
Observed address: "/ip4/23.119.164.150/tcp/53584"
Listen addresses:
	- "/ip4/127.0.0.1/tcp/4001"
	- "/ip4/164.92.118.108/tcp/4001"
	- "/ip4/10.48.0.5/tcp/4001"
	- "/ip4/10.124.0.2/tcp/4001"
Protocols:
	- "/libp2p/circuit/relay/0.2.0/hop"
	- "/ipfs/ping/1.0.0"
	- "/ipfs/id/1.0.0"
	- "/ipfs/id/push/1.0.0"

```

The intent of this repo is to determine what's happeing. I beleive I've narrowed
down the problem as being in async-io. When it's working I see the following in the logs
of release build of relay_v2:
```
[2022-05-03T18:37:52.139518Z TRACE async_io::reactor 475  1] Source::poll_ready:+ dir=0 tick=3 ticks=Some((2, 2))
```
The key here is that tick `3` **IS NOT** in the `ticks` tuple, so `Poll::Ready(OK())`
is returned in the following line:
```
[2022-05-03T18:37:52.139521Z TRACE async_io::reactor 483  1] Source::poll_ready:- dir=0 tick=3 ticks=None Poll::Ready(OK(())
```

Here is that section of the logs:
```
[2022-05-03T18:37:52.139509Z DEBUG libp2p_tcp::provider::async_io 76  1] Tcp::Provider::poll_accept:+ incoming
[2022-05-03T18:37:52.139511Z DEBUG libp2p_tcp::provider::async_io 83  1] Tcp::Provider::poll_accept: call poll_readable cx=Context { waker: Waker { data: 0x55a8a668d970, vtable: 0x55a8a65a72d8 } }
[2022-05-03T18:37:52.139518Z TRACE async_io::reactor 475  1] Source::poll_ready:+ dir=0 tick=3 ticks=Some((2, 2))
[2022-05-03T18:37:52.139521Z TRACE async_io::reactor 483  1] Source::poll_ready:- dir=0 tick=3 ticks=None Poll::Ready(OK(())
[2022-05-03T18:37:52.139523Z DEBUG libp2p_tcp::provider::async_io 85  1] Tcp::Provider::poll_accept: retf poll_readable prr=Ready(Ok(()))
```

When it fails running the debug build relay_v2 I see:
```
[2022-05-03T18:39:43.243471Z TRACE async_io::reactor 475  1] Source::poll_ready:+ dir=0 tick=2 ticks=Some((2, 0))
```
And since tick `2` **IS IN** the `ticks` tuple of `(2,0)` when exiting `Source::poll_ready:-` a `Poll::Pending` is returned:
```
[2022-05-03T18:39:43.243706Z TRACE async_io::reactor 527  1] Source::poll_ready:- dir=0 Poll::Pending at bottom
```

Here is that section of the logs:
```
[2022-05-03T18:39:43.243441Z DEBUG libp2p_tcp::provider::async_io 76  1] Tcp::Provider::poll_accept:+ incoming
[2022-05-03T18:39:43.243458Z DEBUG libp2p_tcp::provider::async_io 83  1] Tcp::Provider::poll_accept: call poll_readable cx=Context { waker: Waker { data: 0x55774859cd90, vtable: 0x5577483cdcc0 } }
[2022-05-03T18:39:43.243471Z TRACE async_io::reactor 475  1] Source::poll_ready:+ dir=0 tick=2 ticks=Some((2, 0))
[2022-05-03T18:39:43.243482Z TRACE async_io::reactor 489  1] Source::poll_ready:  dir=0 was_empty=true
[2022-05-03T18:39:43.243490Z TRACE async_io::reactor 505  1] Source::poll_ready: dir=0 setup new waker and new ticks
[2022-05-03T18:39:43.243497Z TRACE async_io::reactor 94  1] Reactor::ticker:+- val=3
[2022-05-03T18:39:43.243505Z TRACE async_io::reactor 508  1] Source::poll_ready: dir=0 tick=2 ticks=Some((3, 2))
[2022-05-03T18:39:43.243519Z TRACE async_io::reactor 512  1] Source::poll_ready: dir=0 was empty, call poller.modify
[2022-05-03T18:39:43.243529Z TRACE polling::epoll 256  1] ctl:+ MOD epoll_fd=5, fd=4 event_fd=6 { events: 4000201b u64: 0 }
[2022-05-03T18:39:43.243547Z TRACE polling::epoll 183  4] wait: running epoll_fd=5, events.len=1 events.list:
[2022-05-03T18:39:43.243557Z TRACE polling::epoll 188  4] wait: list[0] { events: 1 u64: 0 }
[2022-05-03T18:39:43.243566Z TRACE polling::epoll 192  4] wait: call modify to clear notification epoll_fd=5, res=1
[2022-05-03T18:39:43.243576Z TRACE polling::epoll 199  4] wait: retf modify to clear notification epoll_fd=5, res_rd=Err(Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }) buf=[0, 0, 0, 0, 0, 0, 0, 0]
[2022-05-03T18:39:43.243595Z TRACE polling::epoll 202  4] wait: re-register epoll_fd=5
[2022-05-03T18:39:43.243604Z TRACE polling::epoll 256  4] ctl:+ MOD epoll_fd=5, fd=6 event_fd=6 { events: 4000201b u64: ffffffffffffffff }
[2022-05-03T18:39:43.243618Z TRACE polling::epoll 270  4] ctl:- MOD epoll_fd=5, fd=6 event_fd=6 res=Ok(0)
[2022-05-03T18:39:43.243631Z TRACE polling::epoll 212  4] wait:- epoll_fd=5, res=1
[2022-05-03T18:39:43.243644Z TRACE async_io::reactor 325  4] ReactorLock::react: tick=3 1 I/O events+
[2022-05-03T18:39:43.243657Z TRACE async_io::reactor 330  4] ReactorLock::react: tick=3 ev=Event { key: 0, readable: true, writable: false }
[2022-05-03T18:39:43.243682Z TRACE polling::epoll 270  1] ctl:- MOD epoll_fd=5, fd=4 event_fd=6 res=Ok(0)
[2022-05-03T18:39:43.243695Z TRACE async_io::reactor 523  1] Source::poll_ready: dir=0 was empty, retf poller.modify Ok
[2022-05-03T18:39:43.243706Z TRACE async_io::reactor 527  1] Source::poll_ready:- dir=0 Poll::Pending at bottom
```

Here is the full logs for both debug and release:
- [relay_v2.Debug-relay_v2-ok-fb953b.nightly-1.62.0-de1bc.bt-incoming-connection.debug.trace.1](https://drive.google.com/file/d/1JrJljRLmmIpNu5mqGaAryarXcZUnnycf/view?usp=sharing)
- [relay_v2.Debug-relay_v2-ok-fb953b.nightly-1.62.0-de1bc.bt-incoming-connection.release.trace.1](https://drive.google.com/file/d/1g41NDd_0zqY5SJM5q68lcyyhuIdMmFmu/view?usp=sharing)

And here are Ubuntu 20.04 executables:
- [relay_v2.Debug-relay_v2-ok-fb953b.nightly-1.62.0-de1bc.bt-incoming-connection.debug](https://drive.google.com/file/d/1PhLnhgOng8KZ2oeymtTWRBNTtl8tJmFx/view?usp=sharing)
- [relay_v2.Debug-relay_v2-ok-fb953b.nightly-1.62.0-de1bc.bt-incoming-connection.release](https://drive.google.com/file/d/1L25cfzRW1qRnd29r0lg7TXpn07w-Xsve/view?usp=sharing)

The source for the above executable is tagged with [Debug-relay_v2-ok.v1](https://github.com/winksaville/rust-libp2p/tree/Debug-relay_v2-ok.v1)
and the corresponding async-io is [Add-debug-v11](https://github.com/winksaville/async-io/tree/Add-debug-v11).
I've added a [`[patch.cartes-io]` section to `libp2p/Cargo.toml`](https://github.com/winksaville/rust-libp2p/blob/Debug-relay_v2-ok.v1/Cargo.toml#L175-L178)
which has libp2p use that version of async-io. As you can see I also used special versions of the netlink and polling crates.

## Building and running

You should be able to clone the repo and then build and run with:
```
wink@3900x 22-05-04T18:17:09.631Z:~/prgs/rust/myrepos/debug-async-io (main)
$ RUST_LOG=trace cargo run -- --show-output
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/debug-async-io --show-output`
[2022-05-04T18:17:21.344572Z TRACE debug_async_io 21  1] main:+
Hello, world!
[2022-05-04T18:17:21.344594Z TRACE debug_async_io 25  1] main:-
```

Of note the logged not the default:
```
[2022-05-04T18:17:21.344572Z TRACE debug_async_io 21  1] main:+
``
The data in `[]` is from a custom formatter in `main.rs`:
```
fn env_logger_init() {
    let env = env_logger::Env::default();
    env_logger::Builder::from_env(env).format(|buf, record| {
        let time = std::time::SystemTime::now();
        writeln!(buf, "[{} {:5} {} {} {:2}] {}",
            humantime::format_rfc3339_micros(time),
            record.level(),
            if let Some(s) = record.module_path_static() { s } else { "" },
            if let Some(v) = record.line() { v } else { 0 },
            std::thread::current().id().as_u64(),
            record.args())
    }).init();
}
```
And the first executable line `main` invokes `env_logger_init`:
```
fn main() {
    env_logger_init();
    log::trace!("main:+");

    println!("Hello, world!");

    log::trace!("main:-");
}
```
As you can see in env_logger_init the contents of the header, `[]` is:
- UTC time in rfc3339 with micro second precision, `2022-05-04T18:17:21.344572Z`
- record.level, `TRACE`
- module_path, `debug_async_io`
- line number of the log statement, `21`
- thread id as an integer, `1`

Because I'm using `as_u64()` to output the thread id integer I need
to use an unstable version for the rust compiler and I'm using `nightly`.
To make this the default I've added `rust-toolchain.toml` to the project
with `channel=nightly`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
