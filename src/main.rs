use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

struct RESPDataType;

impl RESPDataType {
    const ARRAY: u8 = b'*';
    const BULK: u8 = b'$';
}

fn evaluate_resp(mut cmd: &[u8]) -> String {
    //  *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n

    let mut len: u8 = 0;
    if cmd[0] == RESPDataType::ARRAY {
        len = cmd[1] - b'0';
        //  $4\r\nECHO\r\n$3\r\nhey\r\n
        cmd = &cmd[4..];
    }

    match cmd[0] {
        RESPDataType::BULK => {
            let args = get_args(cmd, len);
            match &args[0] {
                x if x == "ping" => "+PONG\r\n".to_string(),
                x if x == "echo" => format!("+{}\r\n", args[1]),
                _ => "-not_supported command".to_string(),
            }
        }
        _ => "-not_supported data type\r\n".to_string(),
    }
}

fn get_args(mut cmd: &[u8], mut len: u8) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    while len > 0 {
        // $4\r\nECHO\r\n$3\r\nhey\r\n
        let arg_len: usize = (cmd[1] - b'0') as usize;
        let arg = String::from_utf8_lossy(&cmd[4..arg_len + 4]);
        args.push(arg.into());
        cmd = &cmd[arg_len + 4 + 2..];
        len -= 1;
    }
    args
}

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
    let mut buffer = [0u8; 120];

    while let Ok(_) = stream.read(&mut buffer) {
        let val: String = evaluate_resp(&mut buffer);
        _ = stream.write(&val.as_bytes())
    }
}
