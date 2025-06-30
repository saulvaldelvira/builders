use std::collections::HashMap;
use builders::*;

#[derive(Builder, Debug, PartialEq)]
#[builder(clone)]
pub struct HttpRequest {
    #[builder(def = { String::from("GET") })]
    method: String,
    url: Box<str>,
    #[builder(map = "header")]
    headers: HashMap<Box<str>, Box<str>>,
    #[builder(map = "param")]
    params: HashMap<Box<str>, Box<str>>,
    #[builder(map = "response_header")]
    response_headers: HashMap<Box<str>, Box<str>>,
    #[builder(def = 1.0)]
    version: f32,
    #[builder(def = 200u16)]
    status: u16,
    #[builder(optional = true)]
    body: Option<Box<[u8]>>,
}

pub fn main() {
    let b = HttpRequest::builder()
        .url("Loll")
        .response_header("Content-Type", "text/html")
        .header("Host", "example.com")
        .body([1, 2, 3]);

    let act1 = b.clone().build().unwrap();
    let act2 = b.build().unwrap();

    let mut exp = HttpRequest {
        url: "Loll".into(),
        body: Some(Box::from([1, 2, 3])),
        params: HashMap::new(),
        headers: HashMap::new(),
        method: "GET".into(),
        status: 200,
        version: 1.0,
        response_headers: HashMap::new(),
    };

    exp.response_headers.insert("Content-Type".into(), "text/html".into());
    exp.headers.insert("Host".into(), "example.com".into());

    assert_eq!(act1, exp);
    assert_eq!(act2, exp);
}
