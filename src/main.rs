use std::{
    collections::HashMap,
    env::args,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

struct RESPDataType;

impl RESPDataType {
    const ARRAY: u8 = b'*';
    const BULK: u8 = b'$';
}

fn set_db(
    db: &Mutex<HashMap<String, (String, Instant)>>,
    key: &str,
    value: &str,
    expiry: u64,
) -> String {
    let now = Instant::now();
    let expiry_time = now + Duration::from_millis(expiry);

    if let Ok(mut db) = db.lock() {
        db.insert(key.to_string(), (value.to_string(), expiry_time));
    }

    "+OK\r\n".to_string()
}

fn get_db(db: &Mutex<HashMap<String, (String, Instant)>>, key: &str) -> String {
    if let Ok(db) = db.lock() {
        if let Some(value) = db.get(&key.to_string()) {
            if value.1 < Instant::now() {
                return "$-1\r\n".to_string();
            }
            let result = &value.0;
            return format!("${}\r\n{}\r\n", result.len(), result);
        }
    }
    "$-1\r\n".to_string()
}

fn evaluate_resp(mut cmd: &[u8], db: &Mutex<HashMap<String, (String, Instant)>>) -> String {
    //  *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n

    let mut len: u8 = 0;
    if cmd[0] == RESPDataType::ARRAY {
        let mut digits: Vec<char> = Vec::new();
        let mut i = 1;
        while i < cmd.len() && cmd[i] - b'0' < 10 {
            digits.push(cmd[i].into());
            i += 1;
        }
        let s: String = (&digits).into_iter().collect();
        len = s.parse().expect("not a valid number");
        //  $4\r\nECHO\r\n$3\r\nhey\r\n
        cmd = &cmd[(3 + digits.len())..];
    }

    // println!("{len}");
    match cmd[0] {
        RESPDataType::BULK => {
            let args = get_args(cmd, len);
            // println!("{args:?}");
            match &args[0].to_lowercase()[..] {
                "ping" => "+PONG\r\n".to_string(),
                "echo" => format!("+{}\r\n", args[1]),
                "set" => match len {
                    5 => set_db(db, &args[1], &args[2], args[4].parse::<u64>().unwrap()),
                    _ => set_db(db, &args[1], &args[2], u64::MAX),
                },
                "get" => get_db(db, &args[1]),
                "info" => "+role:master\r\n".to_string(),
                _ => "-not_supported command".to_string(),
            }
        }
        _ => "-not_supported data type\r\n".to_string(),
    }
}

fn get_args(mut cmd: &[u8], mut len: u8) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    while len > 0 {
        let mut digits: Vec<char> = Vec::new();
        let mut i = 1;
        while i < cmd.len() && cmd[i] - b'0' < 10 {
            digits.push(cmd[i].into());
            i += 1;
        }
        let digits_str: String = (&digits).into_iter().collect();
        // $4\r\nECHO\r\n$3\r\nhey\r\n
        let arg_len: usize = digits_str.parse().expect("not a valid args len");
        let start = 3 + digits_str.len();
        let arg = String::from_utf8_lossy(&cmd[start..start + arg_len]);
        args.push(arg.into());
        cmd = &cmd[start + arg_len + 2..];
        len -= 1;
    }
    args
}

fn main() {
    println!("Logs from your program will appear here!");
    let mut port = "6379";
    let args: Vec<_> = args().collect();
    if args.len() > 1 && args[1] == "--port" {
        port = &args[2]
    }

    let listener: TcpListener =
        TcpListener::bind(format!("127.0.0.1:{}", port)).expect("could not bind");
    let db: Mutex<HashMap<String, (String, Instant)>> = Mutex::new(HashMap::new());
    thread::scope(|s| {
        for stream in listener.incoming() {
            let db = &db;
            s.spawn(move || match stream {
                Ok(mut stream) => {
                    println!("Accepted new connection");
                    handle_stream(&mut stream, &db);
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            });
        }
    });
}

fn handle_stream(stream: &mut TcpStream, db: &Mutex<HashMap<String, (String, Instant)>>) {
    let mut buffer = [0u8; 120];

    while let Ok(_) = stream.read(&mut buffer) {
        let val: String = evaluate_resp(&mut buffer, &db);
        _ = stream.write(&val.as_bytes())
    }
}
