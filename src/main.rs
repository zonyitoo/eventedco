extern crate env_logger;
extern crate coroutine;
extern crate eventedco;
#[macro_use]
extern crate log;
extern crate clap;

use std::io::{Read, Write};
use std::thread;

use coroutine::Coroutine;

use clap::{Arg, App};

use eventedco::net::TcpListener;
use eventedco::processor::Processor;

fn main() {
    let matches = App::new("tcp-echo")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Y. T. Chung <zonyitoo@gmail.com>")
            .arg(Arg::with_name("BIND").short("b").long("bind").takes_value(true).required(true)
                    .help("Listening on this address"))
            .arg(Arg::with_name("THREADS").short("t").long("threads").takes_value(true)
                    .help("Number of threads"))
            .get_matches();

    let bind_addr = matches.value_of("BIND").unwrap();
    let num_threads = matches.value_of("THREADS").unwrap_or("1")
                                                 .parse::<usize>()
                                                 .ok()
                                                 .expect("Threads has to be a number");

    env_logger::init().unwrap();

    let acceptor = TcpListener::bind(bind_addr).unwrap();

    let mut threads = Vec::new();
    for _ in 0..num_threads {
        let acceptor = acceptor.try_clone().unwrap();
        let hdl = thread::spawn(move|| {
            let server = Coroutine::spawn(move|| {
                loop {
                    let mut stream = acceptor.accept().unwrap();
                    let coro = Coroutine::spawn(move|| {
                        let mut buffer = [0u8; 1024];
                        loop {
                            match stream.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    stream.write(&buffer[0..n]).unwrap();
                                },
                                Err(err) => {
                                    error!("Read error: {:?}", err);
                                    break;
                                }
                            }
                        }
                    });

                    coro.resume().unwrap();
                }
            });
            server.resume().unwrap();
            Processor::current().run().unwrap();
        });
        threads.push(hdl);
    }

    drop(acceptor);

    for h in threads {
        h.join().unwrap();
    }
}
