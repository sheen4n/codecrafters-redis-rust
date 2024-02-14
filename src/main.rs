use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").expect("could not bind");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    handle_connection(&mut stream);
                });
            }
            Err(e) => {
                println!("error: {e}");
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let bytes_read: usize = stream.read(&mut buffer).expect("read failure");
        if bytes_read == 0 {
            break;
        }
        let response: &str = "+PONG\r\n";
        stream
            .write_all(response.as_bytes())
            .expect("write failure");
    }
}
