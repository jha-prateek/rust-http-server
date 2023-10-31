use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    TextPlain,
    OctetStream,
    Unknown
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::TextPlain => write!(f, "text/plain"),
            ContentType::OctetStream => write!(f, "application/octet-stream"),
            ContentType::Unknown => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HttpStatus {
    Ok,
    Created,
    NotFound
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpStatus::Ok => write!(f, "200 OK"),
            HttpStatus::Created => write!(f, "201 Created"),
            HttpStatus::NotFound => write!(f, "404 Not Found")
        }
    }
}