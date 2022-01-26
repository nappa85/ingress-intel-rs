#![deny(warnings)]
#![deny(missing_docs)]

//! # ingress_intel_rs
//!
//! Ingress Intel API interface in pure Rust

use std::borrow::Cow;
use std::collections::HashMap;

use reqwest::{Client, Method, Request, Response};

use once_cell::sync::{Lazy, OnceCell};

use regex::Regex;

use percent_encoding::percent_decode_str;

use serde_json::{json, value::Value};

use tokio::sync::RwLock;

use tracing::error;

mod tile_key;
use tile_key::TileKey;

/// getEntities endpoint resource
pub mod entities;

/// getPortalDetails endpoint resources
pub mod portal_details;

/// getPlexts endpoint resources
pub mod plexts;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

static INTEL_URLS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<a[^>]+href="([^"]+)""#).unwrap());
static FACEBOOK_LOGIN_FORM: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<form[^>]+data-testid="royal_login_form"[^>]+action="([^"]+?)"[^>]+>([\s\S]+?)</form>"#).unwrap());
static INPUT_FIELDS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<input([^>]+)>"#).unwrap());
static INPUT_ATTRIBUTES: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([^\s="]+)="([^"]+)""#).unwrap());
// static COOKIE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([^=]+)=([^;]+)"#).unwrap());
static API_VERSION: Lazy<Regex> = Lazy::new(|| Regex::new(r#"/jsc/gen_dashboard_(\w+)\.js"#).unwrap());

async fn call(client: &Client, req: Request, cookie_store: &RwLock<HashMap<String, String>>) -> Result<Response, ()> {
    let url = req.url().to_string();
    let res = client.execute(req)
        .await
        .map_err(|e| error!("error receiving response from {}: {}", url, e))?
        .error_for_status()
        .map_err(|e| error!("unsucessfull response from {}: {}", url, e))?;

    let mut lock = cookie_store.write().await;
    res.cookies().for_each(|c| {
        lock.insert(c.name().to_owned(), c.value().to_owned());
    });

    Ok(res)
}

async fn get_cookies(cookie_store: &RwLock<HashMap<String, String>>) -> String {
    let lock = cookie_store.read().await;
    lock.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("; ")
}

async fn facebook_login(client: &Client, username: &str, password: &str, cookie_store: &RwLock<HashMap<String, String>>) -> Result<(), ()> {
    let req = client.request(Method::GET, "https://www.facebook.com/?_fb_noscript=1")
        // .header("Referer", "https://www.google.com/")
        .header("User-Agent", USER_AGENT)
        .build()
        .map_err(|e| error!("error building first facebook request: {}", e))?;

    let body = call(client, req, cookie_store).await?
        .text().await
        .map_err(|e| error!("error encoding response text: {}", e))?;

    let captures = FACEBOOK_LOGIN_FORM.captures(&body).ok_or_else(|| error!("Facebook login form not found"))?;
    let url = format!(
        "https://www.facebook.com{}",
        captures.get(1)
            .and_then(|m| percent_decode_str(&m.as_str().replace("&amp;", "&")).decode_utf8().ok().map(|s| s.to_string()))
            .ok_or_else(|| error!("Facebook login form URL not found\nbody: {}", body))?
    );
    let form = captures.get(2).map(|m| m.as_str()).ok_or_else(|| error!("Facebook login form contents not found"))?;

    let mut fields = Value::Null;
    for m in INPUT_FIELDS.captures_iter(form) {
        if let Some(input) = m.get(1) {
            let (name, value) = INPUT_ATTRIBUTES.captures_iter(input.as_str())
                .fold((None, None), |(mut name, mut value), im| {
                    let key = im.get(1).map(|s| s.as_str());
                    if key == Some("name") {
                        name = im.get(2).map(|s| s.as_str());
                    }
                    else if key == Some("value") {
                        value = im.get(2).map(|s| s.as_str());
                    }
                    (name, value)
                });
            if let Some(key) = name {
                // if key != "_fb_noscript" && key != "sign_up" {
                    fields[key] = Value::from(value.unwrap_or_else(|| ""));
                // }
            }
        }
    }

    fields["email"] = Value::from(username);
    fields["pass"] = Value::from(password);

    let req = client.request(Method::POST, &url)
        // .header("Referer", "https://www.facebook.com/")
        // .header("Origin", "https://www.facebook.com/")
        .header("User-Agent", USER_AGENT)
        .header("Cookie", get_cookies(cookie_store).await)
        .form(&fields)
        .build()
        .map_err(|e| error!("error building second facebook request: {}", e))?;

    let res = call(client, req, cookie_store).await?;
    res.cookies().find(|c| c.name() == "c_user").ok_or_else(|| error!("Facebook login failed"))?;

    Ok(())
}

fn get_tile_keys_around(latitude: f64, longitude: f64) -> Vec<String> {
    let base = TileKey::new(latitude, longitude);

    vec![
        base.to_string(),
        (base + (-1, -1)).to_string(),
        (base + (-1, 0)).to_string(),
        (base + (-1, 1)).to_string(),
        (base + (0, -1)).to_string(),
        (base + (0, 1)).to_string(),
        (base + (1, 0)).to_string(),
        (base + (1, 1)).to_string(),
        (base + (1, -1)).to_string()
    ]
}

/// Represents an Ingress Intel web client login
pub struct Intel<'a> {
    username: Option<&'a str>,
    password: Option<&'a str>,
    client: Cow<'a, Client>,
    cookie_store: RwLock<HashMap<String, String>>,
    api_version: OnceCell<String>,
    csrftoken: OnceCell<String>,
}

impl<'a> Intel<'a> {
    /// creates a new Ingress Intel web client login from existing Client
    pub fn new(client: &'a Client, username: Option<&'a str>, password: Option<&'a str>) -> Self {
        Intel {
            username,
            password,
            client: Cow::Borrowed(client),
            cookie_store: Default::default(),
            api_version: OnceCell::new(),
            csrftoken: OnceCell::new(),
        }
    }

    /// creates a new Ingress Intel web client login
    pub fn build(username: Option<&'a str>, password: Option<&'a str>) -> Self {
        Intel {
            username,
            password,
            client: Cow::Owned(Client::new()),
            cookie_store: Default::default(),
            api_version: OnceCell::new(),
            csrftoken: OnceCell::new(),
        }
    }

    /// adds a cookie to the store
    pub async fn add_cookie<N, V>(&self, name: N, value: V)
    where
        N: ToString,
        V: ToString,
    {
        let mut lock = self.cookie_store.write().await;
        lock.insert(name.to_string(), value.to_string());
    }

    /// adds multiple cookies to the store
    pub async fn add_cookies<I, N, V>(&self, iter: I)
    where
        I: IntoIterator<Item=(N, V)>,
        N: ToString,
        V: ToString,
    {
        let mut lock = self.cookie_store.write().await;
        for (name, value) in iter {
            lock.insert(name.to_string(), value.to_string());
        }
    }

    async fn cookie_exists(&self, cookie: &str) -> bool {
        let lock = self.cookie_store.read().await;
        lock.get(cookie).is_some()
    }

    /// performs login, if necessary
    pub async fn login(&self) -> Result<(), ()> {
        if self.api_version.get().is_some() {
            return Ok(());
        }

        // permits to add intel cookie without generating it everytime
        let url = if !self.cookie_exists("csrftoken").await {
            // permits to add facebook cookie without generating it everytime
            if !self.cookie_exists("c_user").await {
                // login into facebook
                facebook_login(
                    &self.client,
                    self.username.ok_or_else(|| error!("Missing facebok username"))?,
                    self.password.ok_or_else(|| error!("Missing facebook password"))?,
                    &self.cookie_store
                ).await?;
            }

            // retrieve facebook login url
            let req = self.client.request(Method::GET, "https://intel.ingress.com/")
                .build()
                .map_err(|e| error!("error building first intel request: {}", e))?;
            let intel = call(&self.client, req, &self.cookie_store).await?
                .text()
                .await
                .map_err(|e| error!("error encoding first intel response: {}", e))?;
            INTEL_URLS.captures_iter(&intel)
                .map(|m| m.get(1).map(|s| s.as_str()))
                .filter(Option::is_some)
                .map(Option::unwrap)
                .find(|s| s.starts_with("https://www.facebook.com/"))
                .ok_or_else(|| error!("Can't retrieve Intel's Facebook login URL"))?
                .to_owned()
        }
        else {
            String::from("https://intel.ingress.com/")
        };

        let req = self.client.request(Method::GET, url)
            .header("User-Agent", USER_AGENT)
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .build()
            .map_err(|e| error!("error building second intel request: {}", e))?;
        let res = call(&self.client, req, &self.cookie_store).await?;
        let csrftoken = res.cookies().find(|c| c.name() == "csrftoken").map(|c| c.value().to_string()).ok_or_else(|| error!("Can't find csrftoken Cookie"))?;
        self.csrftoken.set(csrftoken).map_err(|_| error!("Can't set csrftoken"))?;
        let intel = res.text()
            .await
            .map_err(|e| error!("error encoding second intel response: {}", e))?;

        let captures = API_VERSION.captures(&intel).ok_or_else(|| error!("Can't find Intel API version"))?;
        let api_version = captures.get(1).map(|m| m.as_str().to_owned()).ok_or_else(|| error!("Can't read Intel API version"))?;
        self.api_version.set(api_version).map_err(|_| error!("Can't set api_version"))?;

        Ok(())
    }

    /// Retrieves entities informations for a given point
    pub async fn get_entities(&self, latitude: f64, longitude: f64) -> Result<entities::IntelResponse, ()> {
        self.login().await?;

        let body = json!({
            "tileKeys": get_tile_keys_around(latitude, longitude),
            "v": self.api_version.get().ok_or_else(|| error!("missing API version"))?,
        });

        let req = self.client.request(Method::POST, "https://intel.ingress.com/r/getEntities")
            .header("Referer", "https://intel.ingress.com/")
            .header("Origin", "https://intel.ingress.com/")
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .header("X-CSRFToken", self.csrftoken.get().ok_or_else(|| error!("missing CSRFToken"))?)
            .json(&body)
            .build()
            .map_err(|e| error!("error building entities request: {}", e))?;

        call(&self.client, req, &self.cookie_store).await?
            .json()
            .await
            .map_err(|e| error!("error deserializing entities response: {}", e))
    }

    /// Retrieves informations for a given portal
    pub async fn get_portal_details(&self, portal_id: &str) -> Result<portal_details::IntelResponse, ()> {
        self.login().await?;

        let body = json!({
            "guid": portal_id,
            "v": self.api_version.get().unwrap(),
        });

        let req = self.client.request(Method::POST, "https://intel.ingress.com/r/getPortalDetails")
            .header("Referer", "https://intel.ingress.com/")
            .header("Origin", "https://intel.ingress.com/")
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .header("X-CSRFToken", self.csrftoken.get().ok_or_else(|| error!("missing CSRFToken"))?)
            .json(&body)
            .build()
            .map_err(|e| error!("error building portal details request: {}", e))?;

        call(&self.client, req, &self.cookie_store).await?
            .json()
            .await
            .map_err(|e| error!("error deserializing portal details response: {}", e))
    }

    /// Retrieves COMM contents
    pub async fn get_plexts(&self, from: [u64; 2], to: [u64; 2], tab: plexts::Tab, min_timestamp_ms: Option<i64>, max_timestamp_ms: Option<i64>) -> Result<plexts::IntelResponse, ()> {
        self.login().await?;

        let body = json!({
            "minLatE6": from[0],
            "minLngE6": from[1],
            "maxLatE6": to[0],
            "maxLngE6": to[1],
            "minTimestampMs": min_timestamp_ms.unwrap_or(-1),
            "maxTimestampMs": max_timestamp_ms.unwrap_or(-1),
            "tab": tab,
            "v": self.api_version.get().unwrap(),
        });

        let req = self.client.request(Method::POST, "https://intel.ingress.com/r/getPlexts")
            .header("Referer", "https://intel.ingress.com/")
            .header("Origin", "https://intel.ingress.com/")
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .header("X-CSRFToken", self.csrftoken.get().ok_or_else(|| error!("missing CSRFToken"))?)
            .json(&body)
            .build()
            .map_err(|e| error!("error building portal details request: {}", e))?;

        call(&self.client, req, &self.cookie_store).await?
            .json()
            .await
            .map_err(|e| error!("error deserializing portal details response: {}", e))
    }
}


#[cfg(test)]
mod tests {
    use super::Intel;

    use std::env;

    use once_cell::sync::Lazy;

    use tracing::info;

    static COOKIES: Lazy<Option<String>> = Lazy::new(|| env::var("COOKIES").ok());
    static USERNAME: Lazy<Option<String>> = Lazy::new(|| env::var("USERNAME").ok());
    static PASSWORD: Lazy<Option<String>> = Lazy::new(|| env::var("PASSWORD").ok());
    static LATITUDE: Lazy<Option<f64>> = Lazy::new(|| env::var("LATITUDE").map(|s| s.parse().expect("LATITUDE must be a float")).ok());
    static LONGITUDE: Lazy<Option<f64>> = Lazy::new(|| env::var("LONGITUDE").map(|s| s.parse().expect("LONGITUDE must be a float")).ok());
    static PORTAL_ID: Lazy<Option<String>> = Lazy::new(|| env::var("PORTAL_ID").ok());

    #[tokio::test]
    async fn login() -> Result<(), ()> {
        tracing_subscriber::fmt::try_init().ok();

        let intel = Intel::build(USERNAME.as_ref().map(|s| s.as_str()), PASSWORD.as_ref().map(|s| s.as_str()));

        if let Some(cookies) = &*COOKIES {
            intel.add_cookies(cookies.split("; ").filter_map(|cookie| {
                let (pos, _) = cookie.match_indices('=').next()?;
                Some((&cookie[0..pos], &cookie[(pos + 1)..]))
            })).await;
        }

        if let (Some(latitude), Some(longitude)) = (*LATITUDE, *LONGITUDE) {
            info!("get_entities {:?}", intel.get_entities(latitude, longitude).await?);
        }
        if let Some(portal_id) = &*PORTAL_ID {
            info!("get_portal_details {:?}", intel.get_portal_details(portal_id.as_str()).await?);
        }

        Ok(())
    }
}
