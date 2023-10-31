use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::str::from_utf8;

pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub version: String,
}

impl RequestInfo {
    fn empty_info() -> RequestInfo {
        RequestInfo {
            method: "".to_string(),
            path: "".to_string(),
            version: "".to_string(),
        }
    }
}

pub struct RequestContext {
    pub request_info: RequestInfo,
    pub path_params: Vec<String>,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl RequestContext {
    // Reading from TcpStream: https://thepacketgeek.com/rust/tcpstream/reading-and-writing/
    pub fn prepare_request(mut stream: &TcpStream) -> RequestContext {

        let mut buff_reader = BufReader::new(&mut stream);
        let received: Vec<u8> = buff_reader.fill_buf().unwrap().to_vec();
        let request_parts = from_utf8(&received).unwrap().split("\r\n\r\n").collect::<Vec<&str>>();

        let header = request_parts[0];
        let body = request_parts[1].to_string();

        let mut request_info: RequestInfo = RequestInfo::empty_info();
        let mut path_params: Vec<String> = Vec::new();
        let mut headers: HashMap<String, String> = HashMap::new();

        for (pos, h) in header.lines().enumerate() {
            if pos == 0 {
                request_info = Self::parse_start_line(h);
                path_params = Self::parse_path_params(request_info.path.as_str());
            } else {
                let parts = h.split(": ").map(|p| p.trim()).collect::<Vec<&str>>();
                let key = parts[0];
                let value = parts[1];
                headers.insert(key.to_string(), value.to_string());
            }
        }

        buff_reader.consume(received.len());

        RequestContext {
            request_info,
            path_params,
            headers,
            body,
        }
    }

    fn parse_start_line(line: &str) -> RequestInfo {
        let start_line = line.split_whitespace().collect::<Vec<&str>>();
        let method = start_line[0].to_string();
        let path = start_line[1].to_string();
        let version = start_line[2].to_string();
        RequestInfo {
            method,
            path,
            version
        }
    }

    // TODO: fix for empty path params and path with multiple tokens
    fn parse_path_params(path: &str) -> Vec<String> {
        path.strip_prefix("/").unwrap()
            .split("/")
            .skip(1)
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
    }

    pub fn request_info(&self) -> String {
        format!("{} {} {}", self.request_info.method, self.request_info.path, self.request_info.version)
    }
}