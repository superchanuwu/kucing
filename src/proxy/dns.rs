use reqwest::Client;
use serde::Deserialize;
use std::net::IpAddr;
use tokio::time::{timeout, Duration};

// Fungsi untuk melakukan DoH query (DNS over HTTPS)
pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = timeout(
        Duration::from_secs(10), // Timeout after 10 seconds
        client
            .post("https://1.1.1.1/dns-query")
            .header("Content-Type", "application/dns-message")
            .header("Accept", "application/dns-message")
            .body(req_wireformat.to_vec())
            .send(),
    )
    .await??;

    if !response.status().is_success() {
        return Err("Failed to resolve DNS using Cloudflare DoH".into());
    }

    Ok(response.bytes().await?.to_vec())
}

// Fungsi untuk menyelesaikan DNS menggunakan Google DoH sebagai fallback
pub async fn resolve_doh_fallback_google(req_wireformat: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = timeout(
        Duration::from_secs(10), // Timeout after 10 seconds
        client
            .post("https://dns.google/dns-query")
            .header("Content-Type", "application/dns-message")
            .header("Accept", "application/dns-message")
            .body(req_wireformat.to_vec())
            .send(),
    )
    .await??;

    if !response.status().is_success() {
        return Err("Failed to resolve DNS using Google DoH".into());
    }

    Ok(response.bytes().await?.to_vec())
}

// Fungsi untuk menyelesaikan DNS query untuk domain
pub async fn resolve(domain: &str) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // Pertama coba Cloudflare DoH
    let query = create_dns_query(domain)?;
    let response = match doh(&query).await {
        Ok(response) => response,
        Err(_) => {
            // Jika Cloudflare DoH gagal, coba Google DoH
            resolve_doh_fallback_google(&query).await?
        }
    };

    // Parse response dan ambil alamat IP
    let ip = parse_dns_response(&response)?;
    Ok(ip)
}

// Fungsi untuk membuat DNS query
fn create_dns_query(domain: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a DNS query (A record for IPv4)
    // This part will need your DNS query implementation, typically creating wire-format query.
    Ok(vec![]) // Placeholder, replace with actual wire-format query
}

// Fungsi untuk mem-parsing respons DNS
fn parse_dns_response(response: &[u8]) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // Implementasi parsing response DNS, misalnya parsing A record atau AAAA record
    Ok(IpAddr::V4("8.8.8.8".parse()?)) // Placeholder, replace with actual parsed IP
}
