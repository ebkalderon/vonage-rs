# vonage-rs

[Vonage](https://www.vonage.com/communications-apis/) (formerly Nexmo) API
bindings for Rust.

This library (`vonage-rs`) is intended to be an idiomatic Rust equivalent of
[`vonage-node-sdk`]. It enables you to quickly add communications functionality
to your application, including sending SMS messages, making voice calls,
text-to-speech, gathering phone number insights, two-factor authentication, and
more.

[`vonage-node-sdk`]: https://github.com/Vonage/vonage-node-sdk

To use this library, a Vonage account is required. If you don't have an account,
you can always [sign up for free][sign-up].

[sign-up]: https://dashboard.nexmo.com/sign-up?utm_source=DEV_REL&utm_medium=github

See [developer.nexmo.com](https://developer.nexmo.com/) for upstream
documentation.

## Example

```rust
use std::error::Error;
use std::io::BufRead;
use std::time::Duration;

use vonage::verify::{Code, CodeLength};
use vonage::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new("<api_key>", "<api_secret>");
    let mut pending = client
        .verify("+1 555-555-555".parse()?, "vonage-rs")?
        .code_length(CodeLength::Six)
        .pin_expiry(Duration::from_secs(5 * 60))
        .send()
        .await?;

    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();

    let matched = loop {
        let code = lines.next().unwrap()?;
        match pending.check(&code).await? {
            Code::Match(matched) => break matched,
            Code::Mismatch(p) => pending = p,
        }
    };

    println!("Code matches! {:?}", matched);

    Ok(())
}
```

## Product support

Below are the non-beta products that `vonage-rs` aims to support, at minimum:

- [ ] [SMS API](https://developer.nexmo.com/messaging/sms/overview) (sending SMS
  messages)
- [x] [Verify (2FA) API](https://developer.nexmo.com/verify/overview) (user
  authentication)
- [ ] [Voice API](https://developer.nexmo.com/voice/overview) (making voice
  calls)

Below are the non-beta nice-to-have products that `vonage-rs` may support later:

- [ ] [Account API](https://developer.nexmo.com/account/overview) (managing your
  own Vonage account balance)
- [ ] [Application API](https://developer.nexmo.com/application/overview)
(managing your Vonage-connected applications)
- [ ] [Number Insight API](https://developer.nexmo.com/number-insight/overview)
  (retrieving phone number validity info)
- [ ] [Video API](https://tokbox.com/developer/) (embeddable video and screen
  sharing capabilities)

## License

`vonage-rs` is free and open source software distributed under the terms of
either the MIT or the Apache 2.0 license, at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
