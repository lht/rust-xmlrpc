extern crate http;
use std::str;
use std::io::net::tcp::TcpStream;
use std::io::{Reader, println};
use http::client::RequestWriter;
use http::headers::request::HeaderCollection;
use http::method::Post;
use http::method::Get;

fn main() {
    let data = ~"hello, world";
    let mut request =
        RequestWriter::<TcpStream>::new(Post, from_str("http://192.168.56.101").unwrap()).unwrap();

    request.headers.content_length = Some(data.len());
    request.write(data.as_bytes());
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err((_request, error)) => fail!(":-( {}", error),
    };
    let body = response.read_to_end().unwrap();
    println(str::from_utf8(body).expect("not utf8"));

}
