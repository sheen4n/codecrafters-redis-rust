use std::{collections::HashMap, io::Read, net::TcpListener, sync::Mutex, thread, time::Instant};

use crate::{redis_handler::RedisHandler, request::Request};

pub struct Server<'server> {
    host: &'server str,
    port: &'server str,
    db: Mutex<HashMap<String, (String, Instant)>>,
}

impl<'server> Server<'server> {
    pub fn new(host: &'server str, port: &'server str) -> Self {
        Self {
            host,
            port,
            db: Mutex::new(HashMap::new()),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn run(&self) {
        println!("Listening on http://{}", self.addr());
        let listener: TcpListener = TcpListener::bind(self.addr()).expect("could not bind");
        loop {
            thread::scope(|s| {
                for stream in listener.incoming() {
                    match stream {
                        Err(e) => println!("Failed to establish a connection: {}", e),
                        Ok(mut stream) => {
                            println!("accepted new connection");
                            s.spawn(move || {
                                let mut buffer = [0u8; 1024];
                                let mut handler = RedisHandler::new();
                                loop {
                                    let bytes = stream.read(&mut buffer).unwrap();
                                    if bytes == 0 {
                                        break;
                                    }
                                    let response = match Request::try_from(&buffer[..]) {
                                        Ok(request) => handler.handle_request(&request, &self.db),
                                        Err(e) => handler.handle_bad_request(&e),
                                    };
                                    if let Err(e) = response.send(&mut stream) {
                                        println!("Failed to send response: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }
            });
        }
    }
}
