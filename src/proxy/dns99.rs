
use worker::fetch;
use anyhow::Result;
use worker::*; // Cloudflare Workers crate
use serde::Deserialize;

pub async fn doh(req_wireformat: &[u8]) -> Result<Vec<u8>> {
    let url = "https://1.1.1.1/dns-query";

    // Create request with correct arguments (only URL and HTTP method)
    let request = Request::new(url, "POST")?;
    let mut req = request.clone();
    req.body(Some(req_wireformat.to_vec()))?;

    // Use fetch to perform the request
    let response = fetch(req).await?.bytes().await?;
    Ok(response.to_vec())
}

pub async fn resolve(domain: &str) -> Result<String> {
    let url = format!(
        "https://dns.google/resolve?name={}&type=A",
        domain
    );

    // Send the GET request for DNS resolution
    let request = Request::new(&url, "GET")?;
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

    // Attempt to resolve AAAA if A fails
    let url_aaaa = format!(
        "https://dns.google/resolve?name={}&type=AAAA",
        domain
    );
    let request_aaaa = Request::new(&url_aaaa, "GET")?;
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
