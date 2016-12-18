use std::fs::File;
use std::io::{Read, Write};
use hyper::client::Client;
use hyper::status::StatusCode;

pub struct GameMetadata {
    downloads: GameDownloads,
    libraries: Vec<Lib>
}

struct Lib {
    name: String,
    downloads: LibDownloads,
    rules: Vec<LibRule>
}

struct LibRule {
    action: LibAction,
    os: Option<Os>
}

enum LibAction {
    allow,
    disallow
}

struct Os {
    name: String
}

struct LibDownloads {
    artifact: LibArtifact
}

struct LibArtifact {
    size: usize,
    sha1: String,
    path: String,
    url: String
}

struct GameDownloads {
    client: GameFile,
    server: GameFile
}

struct GameFile {
    sha1: String,
    size: usize,
    url: String
}

fn download_file(out_file: &mut File, addr: &str, client: &Client) -> Result<(), String> {
    let response = client.get(addr).send();
    match response {
        Ok(res) => {
            if res.status == StatusCode::Ok {
                for byte in res.bytes() {
                    if byte.is_err() {
                        return Err(byte.err().unwrap().to_string());
                    }

                    let write_result = out_file.write(&[byte.unwrap()]);
                    if write_result.is_err() {
                        return Err(write_result.err().unwrap().to_string());
                    }
                }
                return Ok(());
            } else {
                return Err(res.status.to_string());
            }
        },
        Err(err) => {
            return Err(err.to_string());
        }
    }
}

pub fn get_launch_info(client: &Client, version: &str) {
    let mut file = File::create("tmp.json").unwrap();
    let base = "https://s3.amazonaws.com/Minecraft.Download/versions/";
    let mut addr = String::with_capacity(base.len() + 2 * version.len() + 6);

    addr.push_str(base);
    addr.push_str(version);
    addr.push('/');
    addr.push_str(version);
    addr.push_str(".json");

    download_file(&mut file, &addr, client);
}

pub fn get_game_versions(out_file: &mut File, client: &Client) -> Result<(), String> {
   return download_file(out_file, "https://launchermeta.mojang.com/mc/game/version_manifest.json", client);
}

pub fn get_game_jar(out_file: &mut File, client: &Client, version: &str) -> Result<(), String> {
    let base = "https://s3.amazonaws.com/Minecraft.Download/versions/";
    let mut addr = String::with_capacity(base.len() + version.len() * 2 + 5);
    addr.push_str(base);
    addr.push_str(version);
    addr.push('/');
    addr.push_str(version);
    addr.push_str(".jar");

    return download_file(out_file, &addr.as_str(), client);
}