use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::ops::Not;
use std::str::from_utf8;

pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub path_params: Vec<String>,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl RequestContext {
    pub fn prepare_request(mut stream: &TcpStream) -> RequestContext {
        // let mut buffered_reader = BufReader::new(stream);
        // let incoming_request_vec = buffered_reader.lines()
        //     .map(|line| line.unwrap())
        //     .take_while(|line| line.is_empty().not())
        //     .collect::<Vec<_>>();

        let read_buffer = &mut [0u8; 1024];
        stream.read(read_buffer).expect("Unable to read stream");
        let request = from_utf8(read_buffer).unwrap().trim_matches(char::from(0));
        let request_parts= request
            .split("\r\n\r\n").map(|a| a.lines().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let header_parts = request_parts[0]
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>();

        let body = if request_parts[1].is_empty().not() {
            request_parts[1][0].to_string()
        } else { String::from("") };

        let start_line = header_parts[0].split_whitespace().collect::<Vec<&str>>();
        let method = start_line[0].to_string();
        let path = start_line[1].to_string();
        let version = start_line[2].to_string();

        let path_params = path.strip_prefix("/").unwrap()
            .split("/")
            .skip(1)
            .map(|p| p.to_string())
            .collect::<Vec<String>>();

        let mut headers: HashMap<String, String> = HashMap::new();
        header_parts
            .iter()
            .skip(1)
            .for_each(|line| {
                let parts = line.split(": ").map(|p| p.trim()).collect::<Vec<&str>>();
                let key = parts[0];
                let value = parts[1];
                headers.insert(key.to_string(), value.to_string());
            });

        RequestContext {
            method,
            path,
            path_params,
            version,
            headers,
            body
        }
    }

    pub fn request_info(&self) -> String {
        format!("{} {} {}", self.method, self.path, self.version)
    }
}