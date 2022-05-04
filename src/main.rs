#![feature(thread_id_value)]

use std::io::Write;

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

fn main() {
    env_logger_init();
    log::trace!("main:+");

    println!("Hello, world!");

    log::trace!("main:-");
}
