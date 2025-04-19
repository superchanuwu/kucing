use reqwest::Error;
use tokio;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct DNSAnswer {
    data: String,
}

#[derive(Deserialize, Debug)]
struct DoHResponse {
    Answer: Option<Vec<DNSAnswer>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // DNS over HTTPS URL (Cloudflare)
    let url = "https://dns.google/resolve?name=example.com&type=A";

    // Mengirim permintaan GET menggunakan reqwest
    let response = reqwest::get(url).await?;

    // Mengambil body dari respons
    let body = response.text().await?;

    // Menampilkan body dari respons
    println!("Response body: {}", body);

    // Parsing JSON ke dalam struktur Rust
    let parsed: DoHResponse = serde_json::from_str(&body)?;

    // Memeriksa dan menampilkan hasil
    if let Some(answers) = parsed.Answer {
        for ans in answers {
            println!("IP Address: {}", ans.data);
        }
    } else {
        println!("No DNS answers found.");
    }

    Ok(())
}
