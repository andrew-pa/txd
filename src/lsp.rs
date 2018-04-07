
use std::error::Error;
use std::process::*;
use std::sync::{Arc, Mutex, TryLockError, atomic::AtomicUsize, atomic::Ordering};
use std::cell::RefCell;
use std::thread;
use std::io::{self, Read, Write, Error as IOError, ErrorKind as IOErrorKind};
use std::collections::HashMap;
use std::result::Result as SResult;
use toml::Value as TomlValue;
use json;
use json::{JsonValue};
use super::ConfigError;

use futures;
use futures::prelude::*;

use mio;
use mio_named_pipes::NamedPipe;

#[cfg(target_os="windows")]
struct ProcessInPipe(NamedPipe);
#[cfg(target_os="windows")]
struct ProcessOutPipe(NamedPipe);

impl ProcessInPipe {
    fn wrap(ins: ChildStdin) -> ProcessInPipe {
        use std::os::windows::io::*;
        unsafe {
            ProcessInPipe(NamedPipe::from_raw_handle(ins.into_raw_handle()))
        }
    }
}
impl ProcessOutPipe {
    fn wrap(out: ChildStdout) -> ProcessOutPipe {
        use std::os::windows::io::*;
        unsafe {
            ProcessOutPipe(NamedPipe::from_raw_handle(out.into_raw_handle()))
        }
    }
}

impl Read for ProcessOutPipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for ProcessInPipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[derive(Debug)]
enum FutureResponseError {
    LockPoisoned
}

impl Error for FutureResponseError {
    fn description(&self) -> &str {
        match self {
            FutureResponseError::LockPoisoned => "poisoned lock"
        }
    }
}

use std::fmt::*;
impl Display for FutureResponseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.description())
    }
}

struct FutureResponse {
    id: usize,
    response_pool: Arc<Mutex<HashMap<usize, JsonValue>>>
}

impl Future for FutureResponse {
    type Item = JsonValue;
    type Error = FutureResponseError;

    fn poll(&mut self) -> SResult<Async<JsonValue>, FutureResponseError> {
        match self.response_pool.try_lock() {
            Ok(ref mut rp) => {
                rp.remove(&self.id).map_or_else(|| Ok(Async::NotReady), |r| Ok(Async::Ready(r))) 
            },
            Err(TryLockError::Poisoned(_)) => Err(FutureResponseError::LockPoisoned),
            Err(TryLockError::WouldBlock) => Ok(Async::NotReady)
        }
    }
}

pub struct LanguageServer {
    ps: Child,
    response_pool: Arc<Mutex<HashMap<usize, JsonValue>>>,
    request_queue: Arc<Mutex<Vec<JsonValue>>>,
    next_id: Arc<AtomicUsize>,
    response_thread: Option<thread::JoinHandle<()>>,
}

impl LanguageServer {
    pub fn new(config: &TomlValue) -> SResult<LanguageServer, Box<Error>> {
        let mut ps = Command::new(config.get("cmd")
                             .ok_or(ConfigError::Missing("language server command"))?.as_str()
                             .ok_or(ConfigError::Invalid("language server command"))?)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
        let mut ins = ProcessInPipe::wrap(ps.stdin.take().unwrap());
        let mut out = ProcessOutPipe::wrap(ps.stdout.take().unwrap());//.take().as_mut().ok_or(IOError::new(IOErrorKind::NotConnected, "language server child process output not connected")).unwrap();
        let mut ls = LanguageServer {
            ps,
            response_pool: Arc::new(Mutex::new(HashMap::new())),
            request_queue: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(AtomicUsize::new(1)),
            response_thread: None
        };
        let poll = mio::Poll::new().unwrap();
        poll.register(&out.0, mio::Token(0), mio::Ready::readable(), mio::PollOpt::edge()/* | mio::PollOpt::oneshot()*/).unwrap();
        poll.register(&ins.0, mio::Token(1), mio::Ready::writable(), mio::PollOpt::edge()).unwrap();
        let mut rq = ls.request_queue.clone();
        let mut rp = ls.response_pool.clone();
        ls.response_thread = Some(thread::spawn(move || {
            let mut buf: [u8; 512] = [0; 512];
            let mut events = mio::Events::with_capacity(1024);
            loop {
                poll.poll(&mut events, None).unwrap();
                for event in events.iter() {
                    match event.token() {
                        mio::Token(0) => {
                            let n = out.read(&mut buf).unwrap();
                            if n > 0 {
                                println!("read {}:{:?};\n\t ->{}", n, &buf[0..n], String::from_utf8_lossy(&buf[0..n]));
                            }
                            //poll.reregister(&out.0, mio::Token(0), mio::Ready::readable(), mio::PollOpt::level() | mio::PollOpt::oneshot()).unwrap();
                        }
                        mio::Token(1) => {
                            println!("cin writable!");
                            let mut resp = rq.lock().expect("lock request queue");
                            if resp.len() > 0 {
                                let msg_s = json::stringify(resp.pop());
                                let msg_s = format!("Content-Length: {}\r\n\r\n{}\r\n", msg_s.len(), msg_s);
                                println!("sending {}", msg_s);
                                write!(&mut ins, "{}", msg_s);
                                ins.flush().expect("flush ins");
                            }
                        }
                        _ => unreachable!()
                    }
                }
            }
        }));
        let mut req = JsonValue::new_object();
        req["processId"] = json::Null;
        req["rootUri"] = json::Null;
        req["capabilities"] = JsonValue::new_object();
        ls.send("initialize", req).expect("send init");
        Ok(ls)
    }

    fn send<S: AsRef<str>>(&mut self, method: S, params: JsonValue) -> SResult<FutureResponse, Box<Error>> {
        let mut msg = JsonValue::new_object();
        msg["jsonrpc"] = ("2.0").into();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        msg["id"] = id.into();
        msg["method"] = method.as_ref().into();
        msg["params"] = params;

        self.request_queue.lock().expect("lock queue").push(msg);

        Ok(FutureResponse {
            id, response_pool: self.response_pool.clone()
        })
    }
}

impl Drop for LanguageServer {
    fn drop(&mut self) {
        if let Some(t) = self.response_thread.take() {
            t.join().expect("join response thread");
        }
    }
}
