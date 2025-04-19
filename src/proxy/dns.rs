use crate::_worker_fetch::fetch;
use worker::*; // Mengimpor semua API dari worker, termasuk fetch
use anyhow::Result;
use serde::Deserialize;

pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    let url = "https://1.1.1.1/dns-query";

    // Membuat request dengan argumen yang benar (URL dan HTTP method)
    let request = Request::new(url, Method::Post)?; // Gunakan Method::Post
    let mut req = request.clone();
    req.body(Some(req_wireformat.to_vec()))?;

    // Gunakan fetch untuk melakukan request
    let response = fetch(req).await?.bytes().await?;
    Ok(response.to_vec())
}

pub async fn resolve(domain: &str) -> Result<String> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=A",
        domain
    );

    // Kirim request GET untuk resolusi DNS
    let request = Request::new(&url, Method::Get)?; // Gunakan Method::Get
    let resp = fetch(request).await?.text().await?;

    #[derive(Deserialize)]
    struct DoHAnswer {
        Answer: Option<Vec<DNSAnswer>>,
    }

    #[derive(Deserialize)]
    struct DNSAnswer {
        data: String,
    }

    let parsed: DoHAnswer = serde_json::from_str(&resp)?;
    if let Some(answers) = parsed.Answer {
        for ans in answers {
            if ans.data.parse::<std::net::Ipv4Addr>().is_ok() {
                return Ok(ans.data);
            }
        }
    }

    // Coba resolusi AAAA jika A gagal
    let url_aaaa = format!(
        "https://dns.google/resolve?name={}&type=AAAA",
        domain
    );
    let request_aaaa = Request::new(&url_aaaa, Method::Get)?; // Gunakan Method::Get
    let resp_aaaa = fetch(request_aaaa).await?.text().await?;

    let parsed_aaaa: DoHAnswer = serde_json::from_str(&resp_aaaa)?;
    if let Some(answers) = parsed_aaaa.Answer {
        for ans in answers {
            if ans.data.parse::<std::net::Ipv6Addr>().is_ok() {
                return Ok(ans.data);
            }
        }
    }

    Err(anyhow::anyhow!("resolve: no valid A or AAAA record found"))
}
