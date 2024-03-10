mod redis_handler;
mod request;
mod response;
mod server;
use std::env::args;

use server::Server;

fn main() {
    println!("Logs from your program will appear here!");
    let mut port = "6379";
    let args: Vec<_> = args().collect();
    if args.len() > 1 && args[1] == "--port" {
        port = &args[2]
    }
    let server = Server::new("127.0.0.1", port);
    server.run();
}
