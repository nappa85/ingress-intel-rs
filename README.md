# ingress-intel-rs
Ingress Intel API interface in pure Rust

## WARNING
Only Facebook login is supported, there are no plans for Google login support

## Example

```rust
use reqwest::Client;

use ingress_intel_rs::Intel;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let client = Client::new();

    let mut intel = Intel::new(client, Some("your@facebook.email"), Some("your_facebook_password"));
    println!("get_portal_details {:?}", intel.get_portal_details("your_portal_id").await?);

    Ok(())
}
```

## WARNING 2
Facebook often blocks suspect login attempts, a workaround can be to pass directly valid cookie values taken from your browser

## Example 2

```rust
use reqwest::Client;

use ingress_intel_rs::Intel;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let client = Client::new();

    let mut intel = Intel::new(client, None, None);
    // add facebook cookies
    intel.add_cookie("datr", "datr_cookie_value");
    intel.add_cookie("sb", "sb_cookie_value");
    intel.add_cookie("fr", "fr_cookie_value");
    intel.add_cookie("c_user", "c_user_cookie_value");
    intel.add_cookie("xs", "xs_cookie_value");
    intel.add_cookie("spin", "spin_cookie_value");
    println!("get_portal_details {:?}", intel.get_portal_details("your_portal_id").await?);

    Ok(())
}
```

## WARNING 3
Facebook login can fail at any time because they change something on their side, better rely on Intel cookies directly

## Example 3

```rust
use reqwest::Client;

use ingress_intel_rs::Intel;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let client = Client::new();

    let mut intel = Intel::new(client, None, None);
    // add intel cookies
    intel.add_cookie("csrftoken", "csrftoken_cookie_value");
    intel.add_cookie("_ga", "_ga_cookie_value");
    intel.add_cookie("ingress.intelmap.lat", "ingress.intelmap.lat_cookie_value");
    intel.add_cookie("ingress.intelmap.lng", "ingress.intelmap.lng_cookie_value");
    intel.add_cookie("ingress.intelmap.zoom", "ingress.intelmap.zoom_cookie_value");
    intel.add_cookie("sessionid", "sessionid_cookie_value");
    intel.add_cookie("_gid", "_gid_cookie_value");
    println!("get_portal_details {:?}", intel.get_portal_details("your_portal_id").await?);

    Ok(())
}
```
