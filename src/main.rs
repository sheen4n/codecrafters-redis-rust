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
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("Accepted new connection");
                handle_stream(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let response: &str = "+PONG\r\n";
    let mut buffer: [u8; 1024] = [0; 1024];

    while let Ok(_) = stream.read(&mut buffer) {
        _ = stream.write(&response.as_bytes())
    }
}
