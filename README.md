# apns2-rust

A Rust crate for sending push notifications to iOS devices via the [APNS][apns]
http/2 API.

## Usage

Cargo.toml:
```toml
[dependencies]
apns2 = "*"
failure = "*"
```

```rust
extern crate apns2;
extern crate failure;

use failure::Error;

fn send(device_token: String, alert: String) -> Result<(), Error> {
    let apns = apns2::Apns::with_certificate(
        "certs/apns_cert.p12", // Path to p12 certificate + key db file.
        Some("passphrase".to_string()), // Passphraase used for the p12 file.
    )?;
    let notification = NotificationBuilder::new(topic, token)
        .title("title")
        .body("body")
        .sound("somesound.mp3")
        .badge(5)
        .build();
    apns.send(n)?;
    
    Ok(())
}
```

## Client

Sadly, no native http/2 Rust libraries are mature enough to be used, so this
crate currently uses cURL via [rust-curl][rust-curl]. 
Once the ecosystem catches up, a native Rust solution will be used.

## Todo

* Add JWT authentication token support
* Add async tokio implementation with tokio-curl

## License

This library is dual-licensed under Apache and MIT.

Check the license files in this repo for details.

[apns]: https://developer.apple.com/library/content/documentation/NetworkingInternet/Conceptual/RemoteNotificationsPG/APNSOverview.html
[rust-curl]: https://github.com/alexcrichton/curl-rust
