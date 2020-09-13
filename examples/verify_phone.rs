use std::error::Error;
use std::io::BufRead;
use std::time::Duration;

use vonage::verify::{Code, CodeLength};
use vonage::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("VONAGE_API_KEY")?;
    let api_secret = std::env::var("VONAGE_API_SECRET")?;
    let phone_to_verify = std::env::var("PHONE_TO_VERIFY")?;

    let client = Client::new(api_key, api_secret);
    let mut pending = client
        .verify(phone_to_verify.parse()?, "vonage-rs")?
        .code_length(CodeLength::Six)
        .pin_expiry(Duration::from_secs(5 * 60))
        .send()
        .await?;

    println!("Created verify request with ID: {}", pending.request_id());

    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();

    let matched = loop {
        let code = lines.next().unwrap()?;
        match pending.check(&code).await? {
            Code::Match(matched) => break matched,
            Code::Mismatch(p) => {
                eprintln!("Code mismatch! Remaining: {}", p.attempts_remaining());
                pending = p;
            }
        }
    };

    println!("Code matches! {:?}", matched);

    Ok(())
}
