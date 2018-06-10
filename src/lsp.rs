
use std::error::Error;
use std::process::*;
use std::sync::{Arc, Mutex, TryLockError, atomic::AtomicUsize, atomic::Ordering};
use std::cell::RefCell;
use std::thread;
use std::io::{self, Read, Write, Error as IOError, ErrorKind as IOErrorKind};
use std::collections::{HashMap,VecDeque};
use std::result::Result as SResult;
use toml::Value as TomlValue;
use json;
use json::{JsonValue};
use super::ConfigError;

use futures;
use futures::prelude::*;

use mio;
#[cfg(target_os="windows")]
use mio_named_pipes::NamedPipe;

use buffer;

#[cfg(target_os="windows")]
struct ProcessInPipe(NamedPipe);
#[cfg(target_os="windows")]
struct ProcessOutPipe(NamedPipe);

#[cfg(any(target_os="macos", target_os="linux"))]
pub struct Fd<T>(T);

#[cfg(any(target_os="macos", target_os="linux"))]
impl<T: Read> Read for Fd<T> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.0.read(bytes)
    }
}

#[cfg(any(target_os="macos", target_os="linux"))]
impl<T: Write> Write for Fd<T> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[cfg(any(target_os="macos", target_os="linux"))]
impl<T> mio::Evented for Fd<T> where T: ::std::os::unix::prelude::AsRawFd {
    fn register(&self,
                poll: &mio::Poll,
                token: mio::Token,
                interest: mio::Ready,
                opts: mio::PollOpt)
                -> io::Result<()> {
        mio::unix::EventedFd(&self.0.as_raw_fd()).register(poll,
                                                token,
                                                interest | mio::Ready::hup(),
                                                opts)
    }

    fn reregister(&self,
                  poll: &mio::Poll,
                  token: mio::Token,
                  interest: mio::Ready,
                  opts: mio::PollOpt)
                  -> io::Result<()> {
        mio::unix::EventedFd(&self.0.as_raw_fd()).reregister(poll,
                                                  token,
                                                  interest | mio::Ready::hup(),
                                                  opts)
    }

    fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
        mio::unix::EventedFd(&self.0.as_raw_fd()).deregister(poll)
    }
}

#[cfg(any(target_os="macos", target_os="linux"))]
struct ProcessInPipe(Fd<ChildStdin>);
#[cfg(any(target_os="macos", target_os="linux"))]
struct ProcessOutPipe(Fd<ChildStdout>);

impl ProcessInPipe {
    fn wrap(ins: ChildStdin) -> ProcessInPipe {
        #[cfg(target_os="windows")]
        {
            use std::os::windows::io::*;
            unsafe {
                ProcessInPipe(NamedPipe::from_raw_handle(ins.into_raw_handle()))
            }
        }
        #[cfg(any(target_os="macos", target_os="linux"))]
        {
            use std::os::unix::io::*;
            unsafe { ProcessInPipe(Fd(ins)) }
        }
    }
}
impl ProcessOutPipe {
    fn wrap(out: ChildStdout) -> ProcessOutPipe {
        #[cfg(target_os="windows")]
        {
            use std::os::windows::io::*;
            unsafe {
                ProcessOutPipe(NamedPipe::from_raw_handle(out.into_raw_handle()))
            }
        }
        #[cfg(any(target_os="macos", target_os="linux"))]
        {
            use std::os::unix::io::*;
            unsafe { ProcessOutPipe(Fd(out)) }
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
pub enum FutureResponseError {
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

pub struct FutureResponse {
    id: usize,
    response_pool: Arc<Mutex<HashMap<usize, JsonValue>>>
}

impl FutureResponse {
    fn wait(self) -> SResult<JsonValue, FutureResponseError> {
        loop {
            match self.response_pool.try_lock() {
                Ok(ref mut rp) => {
                    match rp.remove(&self.id) {
                        Some(r) => return Ok(r),
                        None => {}
                    }
                },
                Err(TryLockError::Poisoned(_)) => return Err(FutureResponseError::LockPoisoned),
                Err(TryLockError::WouldBlock) => {}
            }
        }
    }
}

impl Future for FutureResponse {
    type Item = JsonValue;
    type Error = FutureResponseError;

    fn poll(&mut self, _: &mut futures::task::Context) -> SResult<Async<JsonValue>, FutureResponseError> {
        match self.response_pool.try_lock() {
            Ok(ref mut rp) => {
                rp.remove(&self.id).map_or_else(|| Ok(Async::Pending), |r| Ok(Async::Ready(r))) 
            },
            Err(TryLockError::Poisoned(_)) => Err(FutureResponseError::LockPoisoned),
            Err(TryLockError::WouldBlock) => Ok(Async::Pending)
        }
    }
}

use std::rc::Rc;

pub struct LanguageServer {
    ps: Child,
    response_pool: Arc<Mutex<HashMap<usize, JsonValue>>>,
    request_queue: Arc<Mutex<VecDeque<JsonValue>>>,
    notification_queue: Arc<Mutex<VecDeque<JsonValue>>>,
    next_id: Arc<AtomicUsize>,
    response_thread: Option<thread::JoinHandle<()>>,
    lang_id: String
}

#[derive(Debug)]
enum ReadState {
    Waiting,
    Data { content_length: usize, data: String }
}

struct JsonRpcHeaderIter<'s> {
    s: &'s str,
    i: usize
}

fn read_headers<'s, S: AsRef<str> + 's>(s: &'s S) -> JsonRpcHeaderIter<'s> {
    JsonRpcHeaderIter { s: s.as_ref(), i: 0 }
}

impl<'s> Iterator for JsonRpcHeaderIter<'s> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.i >= self.s.len() { return None; }
        let s = self.s.split_at(self.i).1;
        match s.find("Content-Length: ") {
            Some(header_start) => {
                let header_end = s.split_at(header_start).1.find("\r\n").unwrap() + header_start + 4;
                //println!("t = {}", &s[header_start+16..header_end-4]);
                let content_length = s[header_start+16..header_end-4].parse::<usize>().expect("parse content length");
                self.i += content_length + header_end;
                Some((content_length,self.i - content_length))
            },
            None => None
        }
    }
}

impl ReadState {
    fn advance<R: Read, F: FnMut(&str)>(self, out: &mut R, buf: &mut [u8], mut done: F) -> ReadState {
        match self {
            ReadState::Waiting => {
                let n = out.read(buf).unwrap();
                let s = String::from_utf8_lossy(&buf[0..n]);

                for (content_length, header_end) in read_headers(&s) {
                    if header_end + content_length > n {
                        // wait for rest of data
                        return ReadState::Data { content_length, data: String::from(s.split_at(header_end).1) };
                    } else {
                        done(&s[header_end..header_end+content_length]);
                    }
                }
                ReadState::Waiting
            },
            ReadState::Data { content_length, mut data } => {
                let n = out.read(buf).unwrap();
                data += &String::from_utf8_lossy(&buf[0..n]);
                
                if data.len() >= content_length {
                    done(&data[0..content_length]);
                    data = data.split_off(content_length);
                }

                for (content_length, header_end) in read_headers(&data) {
                    if header_end + content_length > data.len() {
                        // wait for rest of data
                        return ReadState::Data { content_length, data: String::from(data.split_at(header_end).1) };
                    } else {
                        done(&data[header_end..header_end+content_length]);
                    }
                }
                ReadState::Waiting
            },
        }
    }
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
        let mut out = ProcessOutPipe::wrap(ps.stdout.take().unwrap());
        let mut ls = LanguageServer {
            ps,
            response_pool: Arc::new(Mutex::new(HashMap::new())),
            request_queue: Arc::new(Mutex::new(VecDeque::new())),
            notification_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(AtomicUsize::new(1)),
            response_thread: None,
            lang_id: config.get("language-id").ok_or(ConfigError::Missing("language server id"))?.as_str()
                .ok_or(ConfigError::Invalid("language server id"))?.into()
        };
        let poll = mio::Poll::new().unwrap();
        poll.register(&out.0, mio::Token(0), mio::Ready::readable(), mio::PollOpt::edge()/* | mio::PollOpt::oneshot()*/).unwrap();
        poll.register(&ins.0, mio::Token(1), mio::Ready::writable(), mio::PollOpt::level()).unwrap();
        let rq = ls.request_queue.clone();
        let rp = ls.response_pool.clone();
        let nq = ls.notification_queue.clone();
        ls.response_thread = Some(thread::spawn(move || {
            let mut buf: [u8; 1024] = [0; 1024];
            let mut events = mio::Events::with_capacity(1024);
            let mut read_state_machine = ReadState::Waiting;
            'main: loop {
                match poll.poll(&mut events, None) {
                    Ok(_) => {},
                    Err(e) => match e.kind() {
                        IOErrorKind::BrokenPipe => break 'main,
                        _ => panic!("error polling {:?}", e)
                    }
                };
                for event in events.iter() {
                    match event.token() {
                        mio::Token(0) => {
                            println!("rsm = {:?}", read_state_machine);
                            read_state_machine = read_state_machine.advance(&mut out, &mut buf, |s| {
                                println!("got: {}", s);
                                let j = json::parse(&s).expect("parse response");
                                if let Some(id) = j["id"].as_usize() {
                                    rp.lock().expect("lock response pool").insert(id, j); 
                                } else {
                                    nq.lock().expect("lock notification queue").push_back(j);
                                }
                            });
                        }
                        mio::Token(1) => {
                            //println!("cin writable!");
                            let mut resp = rq.lock().expect("lock request queue");
                            if let Some(msg) = resp.pop_front() {
                                let exiting = msg.has_key("method") && msg["method"] == "exit";
                                let msg_s = json::stringify(msg);
                                let msg_s = format!("Content-Length: {}\r\n\r\n{}", msg_s.len(), msg_s);
                                println!("sending {}", msg_s);
                                write!(&mut ins, "{}", msg_s);
                                ins.flush().expect("flush ins");
                                if exiting {
                                    println!("exiting!");
                                    break 'main;
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                }
            }
        }));
        let init_response = ls.send("initialize", object!{
            "processId" => json::Null,
            "rootUri" => (String::from("file:///") + ::std::env::current_dir().unwrap().to_str().unwrap()),
            "capabilities" => object!{
                "workspace" => object!{
                    "workspaceFolders" => false,
                },
                "textDocument" => object!{
                    "synchronization" => object!{
                        "didSave" => true
                    },
                    "documentSymbol" => object!{
                        "dynamicRegistration" => true
                    }
                }
            }
        }).expect("send init").wait().expect("response from init");
        
        Ok(ls)
    }

    pub fn send<S: AsRef<str>>(&mut self, method: S, params: JsonValue) -> SResult<FutureResponse, Box<Error>> {
        let mut msg = JsonValue::new_object();
        msg["jsonrpc"] = ("2.0").into();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        msg["id"] = id.into();
        msg["method"] = method.as_ref().into();
        msg["params"] = params;

        self.request_queue.lock().expect("lock queue").push_back(msg);

        Ok(FutureResponse {
            id, response_pool: self.response_pool.clone()
        })
    }

    pub fn document_did_open(&mut self, buf: &buffer::Buffer) {
        let lang_id = self.lang_id.clone();
        self.send("textDocument/didOpen", object!{
            "textDocument" => object!{
                "uri" => String::from("file:///") + buf.fs_loc.as_ref().expect("buffer has location").to_str().unwrap(),
                "languageId" => lang_id,
                "version" => buf.version,
                "text" => buf.full_text(),
            }
        }).unwrap();
    }

    pub fn document_did_change(&mut self, buf: &mut buffer::Buffer, changes: Vec<((usize,usize), (usize,usize), usize, &str)>) {
        buf.version += 1;
        self.send("textDocument/didChange", object!{
            "textDocument" => object!{
                "uri" => String::from("file:///") + buf.fs_loc.as_ref().expect("buffer has location").to_str().unwrap(),
                "version" => buf.version
            },
            "contentChanges" => changes.iter().map(|&(start, end, len, text)| object! {
                "range" => object!{
                    "start" => object!{ "line" => start.0, "character" => start.1 },
                    "end"   => object!{ "line" => end.0,   "character" => end.1 },
                },
                "rangeLength" => len,
                "text" => text,
            }).collect::<Vec<_>>()
        }).unwrap();
    }

    pub fn document_did_save(&mut self, buf: &buffer::Buffer) {
        self.send("textDocument/didSave", object!{
            "textDocument" => object!{
                "uri" => String::from("file:///") + buf.fs_loc.as_ref().expect("buffer has location").to_str().unwrap(),
            },
        }).unwrap();
    }

    pub fn document_did_close(&mut self, buf: &buffer::Buffer) {
        self.send("textDocument/didClose", object!{
            "textDocument" => object!{
                "uri" => String::from("file:///") + buf.fs_loc.as_ref().expect("buffer has location").to_str().unwrap(),
            },
        }).unwrap();
    }

    pub fn process_notifications<F: FnMut(&JsonValue)>(&mut self, mut f: F) {
        loop {
            match self.notification_queue.lock().expect("lock notification queue").pop_front() {
                Some(n) => {
                    f(&n)
                }
                None => break
            }
        }
    }
}

impl Drop for LanguageServer {
    fn drop(&mut self) {
        self.send("shutdown", json::Null).expect("send shutdown");
        self.send("exit", json::Null).expect("send exit");
        if let Some(t) = self.response_thread.take() {
            t.join().expect("join response thread");
        }
        self.ps.wait().expect("server terminates"); // some sort of error if not sucessful?
    }
}
