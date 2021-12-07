use anyhow::Result;
use std::collections::HashMap;

fn main() {
    let yaml = clap::load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();
    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap();
    let https = matches.is_present("https");
    if let Some(matches) = matches.subcommand_matches("short") {
        do_short(host, port, https, matches.value_of("url").unwrap()).expect("short sub command");
    } else if let Some(matches) = matches.subcommand_matches("expand") {
        do_expand(host, port, https, matches.value_of("id").unwrap()).expect("expand sub command");
    }
}

fn do_short(host: &str, port: &str, https: bool, req_url: &str) -> Result<()> {
    let url = if https {
        format!("https://{}:{}/short", host, port)
    } else {
        format!("http://{}:{}/short", host, port)
    };
    let mut req = HashMap::new();
    req.insert("url", req_url);
    let resp = send_post(&url, req)?;
    println!("{:?}", resp);
    Ok(())
}

fn do_expand(host: &str, port: &str, https: bool, req_id: &str) -> Result<()> {
    let url = if https {
        format!("https://{}:{}/expand", host, port)
    } else {
        format!("http://{}:{}/expand", host, port)
    };
    let mut req = HashMap::new();
    req.insert("id", req_id);
    let resp = send_post(&url, req)?;
    println!("{:?}", resp);
    Ok(())
}

fn send_post(url: &str, req_json: HashMap<&str, &str>) -> Result<HashMap<String, String>> {
    let clt = reqwest::blocking::Client::new();
    let resp = clt.post(url).json(&req_json).send()?;
    let map_resp: HashMap<String, String> = resp.json()?;
    Ok(map_resp)
}
