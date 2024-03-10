use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    str::{self, FromStr, Utf8Error},
};

#[derive(Debug)]
pub enum RedisCommand {
    PING,
    ECHO,
    SET,
    GET,
    INFO,
}

pub struct RedisCommandError;

impl FromStr for RedisCommand {
    type Err = RedisCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PING" => Ok(Self::PING),
            "ECHO" => Ok(Self::ECHO),
            "SET" => Ok(Self::SET),
            "GET" => Ok(Self::GET),
            "INFO" => Ok(Self::INFO),
            _ => Err(RedisCommandError),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    command: RedisCommand,
    payload: Vec<String>,
    expiry: Option<u64>,
}

impl Request {
    pub fn command(&self) -> &RedisCommand {
        &self.command
    }

    pub fn payload(&self) -> &Vec<String> {
        &self.payload
    }

    pub fn expiry(&self) -> Option<u64> {
        self.expiry
    }
}
struct RESPDataType;

impl RESPDataType {
    const ARRAY: u8 = b'*';
    // const BULK: u8 = b'$';
}

fn get_payload_length(buf: &[u8]) -> i32 {
    let mut payload_length: i32 = 0;
    //  *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
    if buf[0] == RESPDataType::ARRAY {
        let mut digits: Vec<char> = Vec::new();
        let mut i = 1;
        while i < buf.len() && buf[i] != b'\r' && buf[i] != b'\n' && buf[i] - b'0' < 10 {
            digits.push(buf[i].into());
            i += 1;
        }
        let s: String = (&digits).into_iter().collect();
        payload_length = s.parse().expect("not a valid number");
    }
    payload_length
}

fn get_bulk_args(mut buf: &[u8], payload_length: i32) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    for _ in 0..payload_length {
        let mut digits: Vec<char> = Vec::new();
        let mut i = 1;
        while i < buf.len() && buf[i] != b'\r' && buf[i] != b'\n' && buf[i] - b'0' < 10 {
            digits.push(buf[i].into());
            i += 1;
        }
        let digits_str: String = (&digits).into_iter().collect();
        // $4\r\nECHO\r\n$3\r\nhey\r\n
        let arg_len: usize = digits_str.parse().expect("not a valid args len");
        let start = 3 + digits_str.len();
        let arg = String::from_utf8_lossy(&buf[start..start + arg_len]);
        let arg: String = arg.into();
        args.push(arg.clone());
        buf = &buf[start + arg_len + 2..];
    }
    args
}

impl TryFrom<&[u8]> for Request {
    type Error = ParseError;

    fn try_from(mut buf: &[u8]) -> Result<Request, Self::Error> {
        // let request = str::from_utf8(buf)?;
        let payload_length = get_payload_length(&buf);
        //  $4\r\nECHO\r\n$3\r\nhey\r\n
        let digits_len = payload_length.to_string().chars().count();
        buf = &buf[(3 + digits_len)..];
        let bulk_args = get_bulk_args(&buf, payload_length);
        let command: RedisCommand = bulk_args[0].parse()?;
        let expiry = match (&command, payload_length) {
            (&RedisCommand::SET, 5) => bulk_args[4].parse::<u64>().ok(),
            (&RedisCommand::SET, _) => Some(u64::MAX),
            _ => None,
        };
        Ok(Self {
            command,
            payload: bulk_args,
            expiry,
        })
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_value: Utf8Error) -> Self {
        Self::InvalidEncoding
    }
}

impl From<RedisCommandError> for ParseError {
    fn from(_value: RedisCommandError) -> Self {
        Self::InvalidRedisCommand
    }
}

pub enum ParseError {
    // InvalidRequest,
    InvalidEncoding,
    InvalidRedisCommand,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl ParseError {
    fn message(&self) -> &str {
        match &self {
            &Self::InvalidEncoding => "invalid encoding",
            &Self::InvalidRedisCommand => "invalid redis command",
        }
    }
}
impl Error for ParseError {}
