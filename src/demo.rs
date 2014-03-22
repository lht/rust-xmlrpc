extern crate xmlrpc;

use xmlrpc::client::Client;
use xmlrpc::common::{String};

fn main() {
    let mut c = Client::new(~"http://192.168.56.101:11311");
    println!("{}", c.execute(~"getSystemState", &String(~"/")))
}
