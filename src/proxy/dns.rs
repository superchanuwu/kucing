use anyhow::{Result, Context};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct DNSAnswer {
    data: String,
}

#[derive(Deserialize, Debug)]
struct DoHResponse {
    Answer: Option<Vec<DNSAnswer>>,
}

fn fetch_dns_data(url: &str) -> Result<String> {
    // Membuat Client untuk permintaan HTTP (blocking)
    let client = Client::new();

    // Menyiapkan header untuk permintaan
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Membuat permintaan GET dengan header
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .context("Failed to send request")?;

    // Memeriksa apakah permintaan berhasil
    if response.status().is_success() {
        let body = response.text().context("Failed to read response body")?;
        Ok(body)
    } else {
        Err(anyhow::anyhow!("Request failed with status: {}", response.status()).into())
    }
}

fn fetch_http_request(url: &str) -> Result<String> {
    // Membuat Client untuk permintaan HTTP (blocking)
    let client = Client::new();

    // Mengirim permintaan GET
    let response = client
        .get(url)
        .send()
        .context("Failed to send HTTP request")?;

    // Memeriksa apakah permintaan berhasil
    if response.status().is_success() {
        let body = response.text().context("Failed to read HTTP response body")?;
        Ok(body)
    } else {
        Err(anyhow::anyhow!("HTTP Request failed with status: {}", response.status()).into())
    }
}

fn main() -> Result<()> {
    // DNS over HTTPS URL
    let dns_url = "https://dns.google/resolve?name=example.com&type=A";
    
    // Mengambil data DNS dari URL
    let body = fetch_dns_data(dns_url)?;

    // Menampilkan body dari respons DNS
    println!("DNS Response body: {}", body);

    // Parsing JSON ke dalam struktur Rust untuk DNS
    let parsed: DoHResponse = serde_json::from_str(&body)
        .context("Failed to parse DNS response body as JSON")?;

    // Memeriksa dan menampilkan hasil DNS
    if let Some(answers) = parsed.Answer {
        for ans in answers {
            println!("IP Address from DNS: {}", ans.data);
        }
    } else {
        println!("No DNS answers found.");
    }

    // HTTP Request URL
    let http_url = "https://httpbin.org/get";
    
    // Mengambil data dari HTTP request
    let http_response = fetch_http_request(http_url)?;

    // Menampilkan body dari respons HTTP
    println!("HTTP Response body: {}", http_response);

    Ok(())
}
