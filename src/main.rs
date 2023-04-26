use clap::Parser;
use reqwest;
use std::collections::HashMap;
use tide::log::{debug, error, info};
use tide::prelude::*;
use sha2::Sha256;
use hmac::{Hmac, Mac};
type HmacSha256 = Hmac<Sha256>;
use regex::Regex;
// use base64::{Engine as _, engine::general_purpose};

// Configuration
#[derive(Parser)]
struct Cfg {
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    #[arg(short, long, default_value = "http://mattermost/hook")]
    webhook: String,
    #[arg(short, long)]
    secret: Option<String>,
    #[arg(short, long, default_value_t=false)]
    enforce_signature: bool,
}

#[derive(Clone)]
struct State {
    hook: String,
    secret: Option<Vec<u8>>,
    enforce: bool,
}

// JSON messages
#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
}
#[derive(Debug, Deserialize)]
struct Workflow {
    name: String,
    head_branch: String,
    path: String,
    event: String,
    status: String,
    conclusion: Option<String>,
    html_url: String,
}
#[derive(Debug, Deserialize)]
struct Sender {
    login: String,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Payload {
    repository: Repository,
    workflow_run: Option<Workflow>,
    sender: Sender,
}

async fn send_msg(text: &str, hook: &str) -> Result<reqwest::Response, reqwest::Error> {
    debug!("Sending message {}", text);
    let mut msg = HashMap::new();
    msg.insert("text", text);
    let client = reqwest::Client::new();
    let res = client.post(hook).json(&msg).send().await;
    match res {
        Ok(_) => info!("Message sent"),
        Err(_) => error!("Error"),
    }
    return res;
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    env_logger::init();
    let cfg = Cfg::parse();
    let state = State { 
        hook: cfg.webhook,
        secret: match cfg.secret {
            None => None,
            Some(x) => Some(x.as_bytes().to_vec()),
        },
        enforce: cfg.enforce_signature,
    };
    let addr = format!("{}:{}", cfg.address, cfg.port);
    let mut app = tide::with_state(state);
    app.at("/webhook").post(incoming_webhook);
    app.at("/dump").post(dumper);
    app.at("/send").post(sender);
    info!("Starting");
    app.listen(addr).await?;
    info!("Quitting");
    Ok(())
}

fn verify_secret(body: Vec<u8>, secret: &Option<Vec<u8>>, h: Option<&http_types::headers::HeaderValues>) -> Result<bool, tide::Error> {
    if secret.is_none() {
        debug!("Missing secret");
        return Ok(true);
    }
    if h.is_none() {
        debug!("Missing signature, but the secret is set");
        return Ok(false);
    }
    let secret_str= secret.as_ref().unwrap();
    let sigheader = h.unwrap();
    let signature = sigheader.as_str();
    let re = Regex::new(r"sha256=([0-9a-fA-F]+)").unwrap();
    let caps = re.captures(signature);
    if caps.is_none() {
        debug!("Invalid signature received (regex)");
        return Err(tide::Error::from_str(400, "Invalid signature"));
    }
    let hexsig = caps.unwrap().get(1).unwrap().as_str();
    let mut mac = HmacSha256::new_from_slice(secret_str.as_slice()).expect("Invalid secret");
    mac.update(body.as_slice());
    let result = mac.finalize();
    let code_hex = hex::encode(result.into_bytes());
    debug!("CALC: {}, RECV: {}", code_hex, hexsig);
    if code_hex != hexsig {
        debug!("Signature mismatch");
        return Ok(false);
    }
    Ok(true)
}

async fn incoming_webhook(mut req: tide::Request<State>) -> tide::Result {
    let body = req.body_bytes().await?;
    let j: Payload = serde_json::from_slice(&body)?;
    let secret = &req.state().secret;
    if !verify_secret(body, secret, req.header("x-hub-signature-256"))? {
        info!("Signature mismatch!");
        if req.state().enforce {
            return Err(tide::Error::from_str(403, "Signature mismatch"));
        }
    }
    debug!("Processing event from {}/{}", j.sender.login, j.sender.id);
    if j.workflow_run.is_none() {
        debug!("Not a workflow");
        return Ok("Ok (no workflow)\n".into());
    }
    let wf = j.workflow_run.unwrap();
    let resp = format!(
        "Received status '{}' for workflow '{}'\n",
        wf.status, wf.name
    );
    if wf.conclusion.is_none() {
        debug!("Not concluded workflow");
        return Ok("Ok (not concluded workflow)\n".into());
    }
    let conclusion = wf.conclusion.unwrap();
    if conclusion == "success" {
        debug!("Workflow succeded for {}", j.repository.name);
        return Ok(resp.into());
    }
    if wf.head_branch != "main" && wf.head_branch != "master" {
        debug!(
            "Workflow NOT succeded for {}/{}",
            j.repository.name, wf.head_branch
        );
        return Ok(resp.into());
    }
    let msg = format!(
        r#"
Failure building repository {} ({})
Event: {}
Workflow path: {}
Status: {} ({})
Job: {}
"#,
        j.repository.name,
        j.repository.full_name,
        wf.event,
        wf.path,
        wf.status,
        conclusion,
        wf.html_url,
    );
    send_msg(&msg, &req.state().hook).await?;
    Ok(resp.into())
}

async fn dumper(mut req: tide::Request<State>) -> tide::Result {
    let body = req.body_string().await?;
    info!("{}", body);
    Ok("Ok\n".into())
}

async fn sender(req: tide::Request<State>) -> tide::Result {
    let _res = send_msg("Test message, please ignore", &req.state().hook).await?;
    Ok("Message sent".into())
}
