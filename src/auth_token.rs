extern crate serde_json;

use std::io::Read;
use hyper::client::Client;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::status::StatusCode;

#[derive(Deserialize)]
struct Profile {
    id: String,
    name: String,
    legacy: Option<bool>
}

#[derive(Deserialize)]
struct Response {
    accessToken: String,
    clientToken: String,
    selectedProfile: Profile
}

#[derive(Debug)]
pub struct AuthToken {
    access_token: String,
    client_token: String,
    player_name: String,
    uuid: String
}

impl AuthToken {
    pub fn new(username: &str, password: &str, client_token: &Option<String>, client: &Client) -> Result<AuthToken, String> {
        let request = create_auth_request(username, password, &client_token);

        match send_request(&request.as_str(), "authenticate", client) {
            Ok(json) => {
                let response:Response = serde_json::from_str(&json).unwrap();
                return Ok(AuthToken{access_token: response.accessToken,
                    client_token: response.clientToken,
                    player_name: response.selectedProfile.name,
                    uuid: response.selectedProfile.id});
            },
            Err(err) => {
                return Err(err);
            }
        }
    }

    pub fn refresh(&self, client: &Client) -> Result<AuthToken, String> {
        let request = create_refresh_request(&self.access_token.as_str(), &self.client_token.as_str());

        match send_request(&request.as_str(), "refresh", client) {
            Ok(json) => {
                let response: Response = serde_json::from_str(&json).unwrap();
                return Ok(AuthToken{access_token: response.accessToken,
                    client_token: response.clientToken,
                    player_name: self.player_name.clone(),
                    uuid: self.uuid.clone()});
            },
            Err(err) => {
                return Err(err);
            }
        }
    }

    pub fn invalidate(&self, client: &Client) -> Result<(), String> {
        let request = create_inval_request(&self.access_token.as_str(), &self.client_token.as_str());

        return match send_request(&request.as_str(), "invalidate", client) {
            Ok(_) => {
                Ok(())
            },
            Err(err) => {
                Err(err)
            }
        }
    }
}

fn send_request(body: &str, endpoint: &str, client: &Client) -> Result<String, String> {
    let base = "https://authserver.mojang.com/";
    let mut addr = String::with_capacity(endpoint.len() + base.len());
    addr.push_str(base);
    addr.push_str(endpoint);
    let response = client.post(addr.as_str())
            .body(body)
            .header(ContentType(
                    Mime(TopLevel::Application,
                            SubLevel::Json,
                            vec![(Attr::Charset, Value::Utf8)])))
            .send();

    match response {
        Ok(mut res) => {
            let mut body = String::new();
            if res.status != StatusCode::NoContent {
                res.read_to_string(&mut body).expect("Failed to read response from Mojang!");
            }

            if res.status == StatusCode::Ok {
                return Ok(body);
            } else {
                return Err::<String, String>(parse_auth_error(&mut body));
            }
        }
        Err(err) => {
            return Err(err.to_string());
        }
    }
}

fn parse_auth_error(body: &mut String) -> String {
    body.pop();
    body.remove(0);

    return body.split(',').map(|x: &str| {
        let trimmed: String = x.split(":")
                .last()
                .unwrap_or("")
                .chars()
                .skip(1)
                .take_while(|c| *c != '"').
                collect();
        let mut s = String::with_capacity(trimmed.len() + 4);
        s.push_str(trimmed.as_str());
        s.push_str("    ");
        s
    }).collect();
}

fn create_refresh_request(access_token: &str, client_token: &str) -> String {
    let mut request = String::with_capacity(access_token.len() + client_token.len() + 30);

    request.push_str("{\"accessToken\":\"");
    request.push_str(access_token);
    request.push_str("\",\"clientToken\":\"");
    request.push_str(client_token);
    request.push_str("\"}");

    return request;
}

fn create_auth_request(username: &str, password: &str, client_token: &Option<String>) -> String {
    let start = "{\"agent\":{\"name\":\"Minecraft\",\"version\":1},\"username\":\"";

    let mut request: String = match client_token {
        &None => String::with_capacity(start.len() + username.len() + password.len() + 14),
        &Some(ref token) => String::with_capacity(start.len() + username.len() + password.len() + token.len() + 31),
    };

    request.push_str(&start);
    request.push_str(&username);
    request.push_str("\",\"password\":\"");
    request.push_str(&password);
    if client_token.is_some() {
        request.push_str("\",\"clientToken\":\"");
        request.push_str(client_token.as_ref().unwrap().as_str());
    }
    request.push_str("\"}");

    return request;
}

fn create_inval_request(access_token: &str, client_token: &str) -> String {
    let mut request = String::with_capacity(access_token.len() + client_token.len() + 35);

    request.push_str("{\"accessToken\":\"");
    request.push_str(access_token);
    request.push_str("\".\"clientToken\":\"");
    request.push_str(client_token);
    request.push_str("\"}");

    return request;
}