#![feature(proc_macro)]
#[macro_use]
extern crate serde_derive;
extern crate hyper;

mod auth_token;
mod downloader;
use self::hyper::client::Client;
use std::fs::File;

fn main() {
    let client = Client::new();
    downloader::get_launch_info(&client, "1.11")
}
