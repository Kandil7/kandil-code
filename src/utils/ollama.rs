use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct TagItem { name: String }

#[derive(Deserialize)]
struct TagList { models: Vec<TagItem> }

pub async fn is_available() -> Result<bool> {
    let client = Client::new();
    let resp = client.get("http://localhost:11434/api/tags").send().await;
    match resp {
        Ok(r) => Ok(r.status().is_success()),
        Err(_) => Ok(false),
    }
}

pub async fn list_models() -> Result<Vec<String>> {
    let client = Client::new();
    let resp = client.get("http://localhost:11434/api/tags").send().await?;
    if resp.status().is_success() {
        let tags: TagList = resp.json().await?;
        Ok(tags.models.into_iter().map(|t| t.name).collect())
    } else {
        Err(anyhow::anyhow!("status {}", resp.status()))
    }
}

pub async fn pull_model(name: &str) -> Result<()> {
    #[derive(serde::Serialize)]
    struct PullReq { name: String }
    let client = Client::new();
    let resp = client
        .post("http://localhost:11434/api/pull")
        .json(&PullReq { name: name.to_string() })
        .send()
        .await?;
    if resp.status().is_success() { Ok(()) } else { Err(anyhow::anyhow!("status {}", resp.status())) }
}

pub async fn delete_model(name: &str) -> Result<()> {
    #[derive(serde::Serialize)]
    struct DelReq { name: String }
    let client = Client::new();
    let resp = client
        .post("http://localhost:11434/api/delete")
        .json(&DelReq { name: name.to_string() })
        .send()
        .await?;
    if resp.status().is_success() { Ok(()) } else { Err(anyhow::anyhow!("status {}", resp.status())) }
}
