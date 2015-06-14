use std::cell::UnsafeCell;
use std::io;

use mio::util::Slab;
use mio::{Handler, EventLoop, Token, ReadHint, Interest, Evented, PollOpt};

use coroutine::{Coroutine, Handle};

thread_local!(static PROCESSOR: UnsafeCell<Processor> = UnsafeCell::new(Processor::new()));

pub struct Processor {
    event_loop: EventLoop<IoHandler>,
    handler: IoHandler,
}

impl Processor {
    fn new() -> Processor {
        Processor {
            event_loop: EventLoop::new().unwrap(),
            handler: IoHandler::new(1024),
        }
    }

    pub fn current() -> &'static mut Processor {
        PROCESSOR.with(|p| unsafe {
            &mut *p.get()
        })
    }

    #[cfg(any(target_os = "linux",
          target_os = "android"))]
    pub fn wait_event<E: Evented + ::std::os::unix::io::AsRawFd>(&mut self, io: &E, inst: Interest) -> io::Result<()> {
        let cur_hdl = Coroutine::current().clone();
        let token = self.handler.slabs.insert((io.as_raw_fd(), cur_hdl)).unwrap();
        try!(self.event_loop.register_opt(io, token, inst, PollOpt::oneshot()));
        Coroutine::block();
        Ok(())
    }

    #[cfg(any(target_os = "macos",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "ios",
          target_os = "bitrig",
          target_os = "openbsd"))]
    pub fn wait_event<E: Evented>(&mut self, io: &E, inst: Interest) -> io::Result<()> {
        let cur_hdl = Coroutine::current().clone();
        let token = self.handler.slabs.insert(cur_hdl).unwrap();
        try!(self.event_loop.register_opt(io, token, inst, PollOpt::oneshot()));
        Coroutine::block();
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        while self.handler.slabs.count() != 0 {
            try!(self.event_loop.run_once(&mut self.handler));
        }
        Ok(())
    }
}

#[cfg(any(target_os = "macos",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "ios",
          target_os = "bitrig",
          target_os = "openbsd"))]
struct IoHandler {
    slabs: Slab<Handle>,
}

impl IoHandler {
    fn new(size: usize) -> IoHandler {
        IoHandler {
            slabs: Slab::new(size),
        }
    }
}

#[cfg(any(target_os = "macos",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "ios",
          target_os = "bitrig",
          target_os = "openbsd"))]
impl Handler for IoHandler {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, _: &mut EventLoop<IoHandler>, token: Token, _: ReadHint) {
        match self.slabs.remove(token) {
            Some(hdl) => {
                let res = hdl.resume();
                debug!("Readable: Resume resule: {:?}", res);
            },
            None => {}
        }
    }

    fn writable(&mut self, _: &mut EventLoop<IoHandler>, token: Token) {
        match self.slabs.remove(token) {
            Some(hdl) => {
                let res = hdl.resume();
                debug!("Readable: Resume resule: {:?}", res);
            },
            None => {}
        }
    }
}

#[cfg(any(target_os = "linux",
          target_os = "android"))]
struct IoHandler {
    slabs: Slab<(::std::os::unix::io::RawFd, Handle)>,
}

#[cfg(any(target_os = "linux",
          target_os = "android"))]
impl Handler for IoHandler {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, event_loop: &mut EventLoop<IoHandler>, token: Token, _: ReadHint) {
        use mio::Io;
        use std::convert::From;

        match self.slabs.remove(token) {
            Some((fd, hdl)) => {
                let io: Io = From::from(fd);
                event_loop.deregister(&io).unwrap();
                ::std::mem::forget(io);

                let res = hdl.resume();
                debug!("Readable: Resume resule: {:?}", res);
            },
            None => {}
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<IoHandler>, token: Token) {
        use mio::Io;
        use std::convert::From;

        match self.slabs.remove(token) {
            Some((fd, hdl)) => {
                let io: Io = From::from(fd);
                event_loop.deregister(&io).unwrap();
                ::std::mem::forget(io);

                let res = hdl.resume();
                debug!("Readable: Resume resule: {:?}", res);
            },
            None => {}
        }
    }
}
