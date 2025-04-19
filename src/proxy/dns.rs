use anyhow::{Result, anyhow};
use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;
use std::net::Ipv4Addr;

pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/dns-message"),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/dns-message"));
    
    let client = Client::new();
    let response = client
        .post("https://1.1.1.1/dns-query")
        .headers(headers)
        .body(req_wireformat.to_vec())
        .send()
        .await
        .context("Failed to send DNS request")?
        .bytes()
        .await
        .context("Failed to read DNS response")?;

    Ok(response.to_vec())
}

#[derive(Deserialize)]
struct DoHAnswer {
    Answer: Option<Vec<DNSAnswer>>,
}

#[derive(Deserialize)]
struct DNSAnswer {
    data: String,
}

pub async fn resolve(domain: &str) -> Result<String> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=A",
        domain
    );

    let client = Client::new();
    let resp = client.get(&url).send().await?.text().await?;

    let parsed: DoHAnswer = serde_json::from_str(&resp)
        .context("Failed to parse DNS response for A record")?;

    if let Some(answers) = parsed.Answer {
        for ans in answers {
            if let Ok(ip) = ans.data.parse::<Ipv4Addr>() {
                return Ok(ip.to_string());
            }
        }
    }

    // Jika A record gagal, coba resolve AAAA
    let url_aaaa = format!(
        "https://dns.google/resolve?name={}&type=AAAA",
        domain
    );
    let resp_aaaa = client.get(&url_aaaa).send().await?.text().await?;
    let parsed_aaaa: DoHAnswer = serde_json::from_str(&resp_aaaa)
        .context("Failed to parse DNS response for AAAA record")?;

    if let Some(answers) = parsed_aaaa.Answer {
        for ans in answers {
            if let Ok(ip) = ans.data.parse::<std::net::Ipv6Addr>() {
                return Ok(ip.to_string());
            }
        }
    }

    Err(anyhow!("resolve: no valid A or AAAA record found"))
}
