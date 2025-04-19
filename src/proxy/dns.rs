
use anyhow::{Result, anyhow, Context}; // Menambahkan import Context
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;
use std::net::Ipv4Addr;
use tokio::time::{sleep, Duration};

// Pastikan impor console_error sudah benar
use worker::console_error;

pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/dns-message"),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/dns-message"));
    
    let client = Client::new();
    let retry_count = 3;

    for _ in 0..retry_count {
        let response = client
            .post("https://1.1.1.1/dns-query")
            .headers(headers.clone())
            .body(req_wireformat.to_vec())
            .send()
            .await
            .context("Failed to send DNS request")?
            .bytes()
            .await
            .context("Failed to read DNS response")?;

        if !response.is_empty() {
            return Ok(response.to_vec());
        } else {
            console_error!("Received empty response, retrying...");
            sleep(Duration::from_secs(1)).await;
        }
    }

    Err(anyhow!("Failed to get valid response after retrying"))
}

#[derive(Deserialize)]
struct DoHAnswer {
    answer: Option<Vec<DNSAnswer>>, // Mengganti Answer menjadi answer
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

    if let Some(answers) = parsed.answer {  // Mengganti Answer menjadi answer
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

    if let Some(answers) = parsed_aaaa.answer {  // Mengganti Answer menjadi answer
        for ans in answers {
            if let Ok(ip) = ans.data.parse::<std::net::Ipv6Addr>() {
                return Ok(ip.to_string());
            }
        }
    }

    Err(anyhow!("resolve: no valid A or AAAA record found"))
}
