# ingress-intel-rs
Ingress Intel API interface in pure Rust

## WARNING
Only Facebook login is supported, there are no plans for Google login support

## Example

```rust
use hyper::{client::Client, Body};

use hyper_tls::HttpsConnector;

use ingress_intel_rs::Intel;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let https = HttpsConnector::new().unwrap();
    let client = Client::builder().build::<_, Body>(https);

    let mut intel = Intel::new(client, "your@facebook.email", "your_facebook_password");
    println!("get_portal_details {:?}", intel.get_portal_details("your_portal_id").await?);

    Ok(())
}
```
