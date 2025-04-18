
use reqwest::Client;
use trust_dns_proto::op::{Query, ResponseCode};
use trust_dns_proto::rr::{RData, RecordType};
use trust_dns_proto::xfer::DnsRequestOptions;
use trust_dns_client::client::SyncClient;
use trust_dns_client::proto::dnssec::{DnsSecAlgorithm, DnsSecMode};
use std::str;

const DOH_SERVER: &str = "https://cloudflare-dns.com/dns-query";  // Cloudflare DoH server

// Fungsi untuk mengirimkan query DNS over HTTPS (DoH)
pub async fn doh(query: &[u8]) -> Result<Vec<u8>, reqwest::Error> {
    let client = Client::new();
    let res = client
        .get(DOH_SERVER)
        .header("Accept", "application/dns-message")
        .body(query.to_vec())
        .send()
        .await?;
    Ok(res.bytes().await?.to_vec())
}

// Fungsi untuk menyelesaikan DNS menggunakan DoH
pub async fn resolve_doh(host: &str) -> Result<String, String> {
    let query = create_dns_query(host);  // Buat query DNS
    let response = doh(&query).await.map_err(|e| e.to_string())?;
    
    // Parse response dan dapatkan alamat IP
    let ip = parse_dns_response(&response)?;
    Ok(ip)
}

// Fungsi untuk membuat query DNS
fn create_dns_query(host: &str) -> Vec<u8> {
    // Membuat query DNS untuk mencari record A (IPv4) atau AAAA (IPv6)
    let mut query = Vec::new();
    // [Kode untuk membangun query DNS sesuai dengan standar DNS]
    query
}

// Fungsi untuk memparsing DNS response dan mendapatkan alamat IP
fn parse_dns_response(response: &[u8]) -> Result<String, String> {
    // Parse response DNS dan ambil alamat IP dari record
    let parsed_response = trust_dns_proto::op::Message::from_vec(response).map_err(|e| e.to_string())?;
    
    if parsed_response.header().response_code() != ResponseCode::NoError {
        return Err("DNS query failed".to_string());
    }

    let ip = parsed_response.answers().iter().find_map(|record| match record.rdata() {
        RData::A(a) => Some(a.to_string()),  // IPv4 address
        RData::AAAA(a) => Some(a.to_string()), // IPv6 address
        _ => None,
    }).ok_or("No valid IP address found".to_string())?;

    Ok(ip)
}

// Fungsi untuk mengekstrak SNI dari TLS ClientHello
pub fn extract_sni_from_tls_client_hello(client_hello: &[u8]) -> Option<String> {
    // [Kode untuk mengekstrak SNI dari TLS ClientHello]
    Some("example.com".to_string())  // Return extracted SNI (contoh)
}
