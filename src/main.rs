use clap::Parser;
use reqwest;
use std::collections::HashMap;
use tide::log::{debug, error, info};
use tide::prelude::*;

// Configuration
#[derive(Parser)]
struct Cfg {
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    #[arg(short, long, default_value = "http://mattermost/hook")]
    webhook: String,
}

#[derive(Clone)]
struct State {
    hook: String,
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
    let state = State { hook: cfg.webhook };
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

async fn incoming_webhook(mut req: tide::Request<State>) -> tide::Result {
    let j: Payload = req.body_json().await?;
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
