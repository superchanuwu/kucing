
use anyhow::Result;
use worker::fetch;
use worker::*; // Cloudflare Workers crate
use serde::Deserialize;

pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    let url = "https://1.1.1.1/dns-query";

    let request = Request::new(url, "POST", Some(req_wireformat.to_vec()), None);
    let response = fetch(request).await?.bytes().await?;

    Ok(response.to_vec())
}

pub async fn resolve(domain: &str) -> Result<String> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=A",
        domain
    );

    let request = Request::new(&url, "GET", None, None);
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

    // Coba resolve AAAA jika A gagal
    let url_aaaa = format!(
        "https://dns.google/resolve?name={}&type=AAAA",
        domain
    );
    let request_aaaa = Request::new(&url_aaaa, "GET", None, None);
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
