extern crate env_logger;
extern crate coroutine;
extern crate eventedco;
#[macro_use]
extern crate log;
extern crate clap;
extern crate rand;

use std::io::{Read, Write};
use std::str;

use coroutine::Coroutine;

use clap::{Arg, App};

use eventedco::net::TcpStream;
use eventedco::processor::Processor;

use rand::Rng;

fn main() {
    let matches = App::new("tcp-echo")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Y. T. Chung <zonyitoo@gmail.com>")
            .arg(Arg::with_name("ADDR").short("a").long("addr").takes_value(true).required(true)
                    .help("Connect to this address"))
            .arg(Arg::with_name("RAND_BYTES").short("b").long("rand-bytes").takes_value(true).required(true)
                    .help("Random bytes len"))
            .get_matches();

    let connect_addr = matches.value_of("ADDR").unwrap().to_owned();
    let byte_len = matches.value_of("RAND_BYTES").unwrap()
                            .parse::<usize>()
                            .ok()
                            .expect("Random bytes has to be an integer");

    env_logger::init().unwrap();

    Coroutine::spawn(move|| {
        let mut stream = TcpStream::connect(&connect_addr[..]).unwrap();

        let mut bytes = Vec::with_capacity(byte_len);
        unsafe { bytes.set_len(byte_len); }

        rand::weak_rng().fill_bytes(&mut bytes);

        stream.write_all(&bytes).unwrap();

        let mut received = Vec::new();
        while received.len() < bytes.len() {
            let mut buf = [0u8; 1024];
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    received.write_all(&buf[..n]).unwrap();
                },
                Err(err) => panic!("{:?}", err),
            }
        }

        assert_eq!(bytes, received);
    }).resume().unwrap();

    Processor::current().run().unwrap();
}
