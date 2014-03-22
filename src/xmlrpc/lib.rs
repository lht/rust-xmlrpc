#[crate_id = "xmlrpc"];
#[crate_type = "rlib"];
#[crate_type = "dylib"];

#[feature(macro_rules)];

extern crate collections;
extern crate serialize;
extern crate url;
extern crate http;

pub mod common;
pub mod client;
