use std::io::Write;
use std::net::{Shutdown, TcpListener};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).expect("TODO: panic message");
                _stream.shutdown(Shutdown::Both).expect("TODO: panic message");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}