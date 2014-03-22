#[allow(dead_code)];
#[allow(unused_variable)];

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use url;
use url::Url;
use http::client::RequestWriter;
use http::method::{Post};
use http::headers::host::Host;
use std::u16;

use common::{Value, Array};
use common::Encoder;

static REQUEST_BEGIN          : &'static str =
    "<?xml version=\"1.0\"?>\r\n<methodCall><methodName>";
static REQUEST_END_METHODNAME : &'static str = "</methodName>\r\n";
static PARAMS_TAG             : &'static str = "<params>";
static PARAMS_ETAG            : &'static str = "</params>";
static PARAM_TAG              : &'static str = "<param>";
static PARAM_ETAG             : &'static str = "</param>";
static REQUEST_END            : &'static str = "</methodCall>\r\n";
static METHODRESPONSE_TAG     : &'static str = "<methodResponse>";
static FAULT_TAG              : &'static str = "<fault>";
static XMLRPC_VERSION         : &'static str = "XMLRPC-rs 0.1";

pub struct Client {
    url: ~Url,
}


impl Client {
    pub fn new(url: ~str) -> Client {
        let url_ = url::from_str(url).unwrap();
        let c = Client {url: ~url_};
        c
    }

    pub fn execute(&mut self, method: ~str, params: &Value) ->
       IoResult<~str> {
        let body = self.mk_request(method, params);

        // make a POST request setup header
        let url_ = *(self.url.clone());
        let mut request = RequestWriter::<TcpStream>::new(Post, url_).unwrap();
        request.headers.content_length = Some(body.len());
        let port =
            u16::parse_bytes(self.url.port.clone().unwrap().as_bytes(), 10);
        request.headers.host =
            Some(Host{name: self.url.host.clone(), port: port});

        request.write(body.as_bytes()).unwrap();
        let mut response = request.read_response().unwrap();
        println!("{}", body);
        response.read_to_str()
    }

    fn mk_request(&mut self, method: ~str, params: &Value) -> ~str {
        let mut body = ~"";

        body.push_str(REQUEST_BEGIN);
        body.push_str(method);
        body.push_str(REQUEST_END_METHODNAME);

        body.push_str(PARAMS_TAG);
        match *params {
            Array(ref es) => {
                for e in es.iter() {
                    body.push_str(PARAM_TAG);
                    body.push_str(Encoder::str_encode(e));
                    body.push_str(PARAM_ETAG);
                }
            }
            _ => {
                body.push_str(PARAM_TAG);
                body.push_str(Encoder::str_encode(params));
                body.push_str(PARAM_ETAG);
            }
        }

        body.push_str(PARAMS_ETAG);
        body.push_str(REQUEST_END);
        body
    }

}

