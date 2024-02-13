// Uncomment this block to pass the first stage
// use std::net::TcpListener;

use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:6379").expect("could not bind");
    for stream in listener.incoming() {
        let mut stream = stream.expect("failed to accept");
        handle_connection(&mut stream);
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer).expect("read failure");
        if bytes_read == 0 {
            break;
        }
        let response = "+PONG\r\n";
        stream.write_all(response.as_bytes()).expect("write failure");
    }
}