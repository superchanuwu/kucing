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

fn process_wire_format(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    // Proses data byte, misalnya mengonversi slice ke Vec<u8>
    // Contoh ini hanya menyalin data ke dalam Vec<u8>
    let processed_data: Vec<u8> = req_wireformat.to_vec();

    // Kembalikan hasil pemrosesan
    Ok(processed_data)
}

fn fetch_dns_data(url: &str) -> Result<Vec<u8>> {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Mengirimkan permintaan GET dan mengembalikan data dalam bentuk raw bytes (Vec<u8>)
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .context("Failed to send DNS request")?;

    // Memeriksa status dari response dan mengambil body sebagai byte array
    if response.status().is_success() {
        let body = response.bytes().context("Failed to read DNS response as bytes")?;
        Ok(body.to_vec()) // Mengembalikan body sebagai Vec<u8>
    } else {
        Err(anyhow::anyhow!("DNS Request failed with status: {}", response.status()).into())
    }
}

fn fetch_http_request(url: &str) -> Result<Vec<u8>> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .context("Failed to send HTTP request")?;

    if response.status().is_success() {
        let body = response.bytes().context("Failed to read HTTP response as bytes")?;
        Ok(body.to_vec()) // Mengembalikan body sebagai Vec<u8>
    } else {
        Err(anyhow::anyhow!("HTTP Request failed with status: {}", response.status()).into())
    }
}

fn main() -> Result<()> {
    // DNS over HTTPS URL
    let dns_url = "https://dns.google/resolve?name=example.com&type=A";
    
    // Mengambil data DNS dari URL sebagai raw bytes
    let body = fetch_dns_data(dns_url)?;

    // Menampilkan body dari respons DNS sebagai raw bytes
    println!("DNS Response body (raw bytes): {:?}", body);

    // Memproses data wireformat (menyalin data ke Vec<u8>)
    let processed_data = process_wire_format(&body)?;
    println!("Processed DNS data: {:?}", processed_data);

    // Parsing JSON ke dalam struktur Rust untuk DNS
    let body_str = String::from_utf8_lossy(&body);  // Mengubah raw bytes menjadi string untuk parsing JSON
    let parsed: DoHResponse = serde_json::from_str(&body_str)
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
    
    // Mengambil data dari HTTP request sebagai raw bytes
    let http_response = fetch_http_request(http_url)?;

    // Menampilkan body dari respons HTTP sebagai string (untuk debugging)
    let http_body_str = String::from_utf8_lossy(&http_response);
    println!("HTTP Response body (raw bytes): {}", http_body_str);

    Ok(())
}
