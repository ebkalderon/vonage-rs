use std::error::Error;
use std::time::Duration;

use vonage::{verify::CodeLength, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("VONAGE_API_KEY")?;
    let api_secret = std::env::var("VONAGE_API_SECRET")?;
    let phone_to_verify = std::env::var("PHONE_TO_VERIFY")?;

    let client = Client::new(api_key, api_secret);
    let pending = client
        .verify(phone_to_verify.parse()?, "vonage-rs")?
        .code_length(CodeLength::Six)
        .pin_expiry(Duration::from_secs(5 * 60))
        .send()
        .await?;

    println!("Created verify request with ID: {}", pending.request_id());

    match pending.check("123456").await {
        Ok(verified) => println!("Code matches! {:?}", verified),
        Err(e) => Err(format!("Verification failed: {}", e))?,
    }

    Ok(())
}
