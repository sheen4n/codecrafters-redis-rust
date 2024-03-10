use std::io::Result as IoResult;
use std::io::Write;

pub struct Response {
    payload: String,
}

impl Response {
    pub fn new(s: String) -> Self {
        Response { payload: s }
    }

    pub fn send(&self, stream: &mut impl Write) -> IoResult<()> {
        write!(stream, "{}", self.payload)
    }
}
