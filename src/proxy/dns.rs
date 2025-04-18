use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use std::net::IpAddr;
use trust_dns_proto::op::{Message, Query};
use trust_dns_proto::rr::{Name, RData, RecordType};
use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/dns-message"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/dns-message"));

    let client = Client::new();

    let response = client
        .post("https://cloudflare-dns.com/dns-query")
        .headers(headers)
        .body(req_wireformat.to_vec())
        .send()
        .await?
        .bytes()
        .await?;

    Ok(response.to_vec())
}

pub async fn resolve_doh(host: &str) -> Result<IpAddr> {
    if let Ok(ip) = resolve_record(host, RecordType::A).await {
        return Ok(ip);
    }

    resolve_record(host, RecordType::AAAA).await
}

async fn resolve_record(host: &str, rtype: RecordType) -> Result<IpAddr> {
    let name = Name::from_ascii(host)?;
    let mut message = Message::new();
    message.add_query(Query::query(name, rtype));
    message.set_id(rand::random());
    message.set_recursion_desired(true);

    let mut buf = Vec::with_capacity(512);
    let mut encoder = BinEncoder::new(&mut buf);
    message.emit(&mut encoder)?;

    let response = doh(&buf).await?;
    let msg = Message::from_vec(&response)?;

    for answer in msg.answers() {
        if let Some(data) = answer.data() {
            match data {
                RData::A(ipv4) => return Ok(IpAddr::V4(*ipv4)),
                RData::AAAA(ipv6) => return Ok(IpAddr::V6(*ipv6)),
                _ => {}
            }
        }
    }

    Err(anyhow::anyhow!("No {:?} record found for {}", rtype, host))
}

pub fn extract_sni_from_tls_client_hello(buf: &[u8]) -> Option<String> {
    if buf.len() < 5 || buf[0] != 0x16 { return None; }
    let mut i = 43;
    if i >= buf.len() { return None; }

    let session_id_len = *buf.get(i)? as usize;
    i += 1 + session_id_len;

    let cipher_suites_len = u16::from_be_bytes([*buf.get(i)?, *buf.get(i+1)?]) as usize;
    i += 2 + cipher_suites_len;

    let compression_methods_len = *buf.get(i)? as usize;
    i += 1 + compression_methods_len;

    let extensions_len = u16::from_be_bytes([*buf.get(i)?, *buf.get(i+1)?]) as usize;
    i += 2;
    let end = i + extensions_len;

    while i + 4 <= end && i + 4 <= buf.len() {
        let ext_type = u16::from_be_bytes([buf[i], buf[i + 1]]);
        let ext_len = u16::from_be_bytes([buf[i + 2], buf[i + 3]]) as usize;
        i += 4;

        if ext_type == 0x00 {
            let sni_len = u16::from_be_bytes([buf[i + 2], buf[i + 3]]) as usize;
            let sni_start = i + 5;
            let sni_end = sni_start + sni_len;
            if sni_end <= buf.len() {
                return Some(String::from_utf8_lossy(&buf[sni_start..sni_end]).to_string());
            }
        }

        i += ext_len;
    }

    None
}