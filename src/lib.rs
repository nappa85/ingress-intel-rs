#![deny(warnings)]
#![deny(missing_docs)]

//! # ingress_intel_rs
//!
//! Ingress Intel API interface in pure Rust

use std::collections::HashMap;
use std::fmt::Display;

use futures_util::TryStreamExt;

use hyper::{service::Service, Request, Body, Response};

use serde::de::DeserializeOwned;

use once_cell::sync::Lazy;

use regex::Regex;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use log::error;

use serde_json::json;

mod tile_key;
use tile_key::TileKey;

/// getEntities endpoint resource
pub mod entities;

/// getPortalDetails endpoint resources
pub mod portal_details;

static INTEL_URLS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<a[^>]+href="([^"]+)""#).unwrap());
static FACEBOOK_LOGIN_FORM: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<form[^>]+action="([^"]+)"[^>]+id="login_form"[^>]*>([\s\S]+)</form>"#).unwrap());
static INPUT_FIELDS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<input[^>]+name="([^"]+)"[^>]*(value="([^"]+)")?"#).unwrap());
static COOKIE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([^=]+)=([^;]+)"#).unwrap());
static API_VERSION: Lazy<Regex> = Lazy::new(|| Regex::new(r#"/jsc/gen_dashboard_(\w+)\.js"#).unwrap());

async fn call_and_deserialize<D, C, E>(client: &mut C, method: &str, url: &str, headers: Option<HashMap<&str, String>>, body: Option<String>, cookies_jar: Option<&mut HashMap<String, String>>) -> Result<D, ()>
where D: DeserializeOwned,
    C: Service<Request<Body>, Response=Response<Body>, Error=E>,
    E: Display,
{
    let res = call(client, method, url, headers, body, cookies_jar).await?;
    serde_json::from_str(&res).map_err(|e| error!("error while decoding response from {}: {}\nbody: {}", url, e, res))
}

async fn call<C, E>(client: &mut C, method: &str, url: &str, headers: Option<HashMap<&str, String>>, body: Option<String>, mut cookies_jar: Option<&mut HashMap<String, String>>) -> Result<String, ()>
where C: Service<Request<Body>, Response=Response<Body>, Error=E>,
    E: Display,
{
    let mut method = method;
    let mut url = url.to_string();
    let mut body = body;
    loop {
        let mut builder = Request::builder()
            .method(method).uri(&url);
        if let Some(ref jar) = cookies_jar {
            builder = builder.header("Cookie", jar.iter().map(|(key, value)| format!("{}={}", key, value)).collect::<Vec<String>>().join("; "));
        }
        if let Some(ref h) = headers {
            for (name, value) in h {
                builder = builder.header(*name, value);
            }
        }
        let req = builder.body(if let Some(b) = body { Body::from(b) } else { Body::empty() }).map_err(|e| error!("error building request to {}: {}", url, e))?;

        let res = client.call(req).await.map_err(|e| error!("error receiving response from {}: {}", url, e))?;
        let success = res.status().is_success();
        let redirect = res.status().is_redirection();
        let (head, stream) = res.into_parts();
        let chunks = stream.map_ok(|b| b.to_vec()).try_concat().await.map_err(|e| error!("error while reading response from {}: {}", url, e))?;
        let res_body = String::from_utf8(chunks.to_vec()).map_err(|e| error!("error while encoding response from {}: {}", url, e))?;
        if success || redirect {
            if let Some(ref mut jar) = cookies_jar {
                head.headers.get_all("Set-Cookie").into_iter().for_each(|c| {
                    if let Ok(s) = c.to_str() {
                        if let Some(captures) = COOKIE.captures(s) {
                            match (captures.get(1), captures.get(2)) {
                                (Some(key), Some(value)) => {
                                    jar.insert(key.as_str().to_string(), value.as_str().to_string());
                                },
                                _ => {},
                            }
                        }
                    }
                });
            }

            if success {
                return Ok(res_body);
            }
            else {
                if let Some(location) = head.headers.get("Location") {
                    method = "GET";
                    let location = location.to_str().map_err(|e| error!("Location header decode error: {}", e))?;
                    if location.starts_with("http") {
                        url = location.to_string();
                    }
                    else {
                        url = format!("{}{}", url.split("/").take(3).collect::<Vec<&str>>().join("/"), location);
                    }
                    body = None;
                }
                else {
                    error!("Locationless redirect");
                    return Err(());
                }
            }
        }
        else {
            error!("unsucessfull response from {}: {:?}\nbody: {}", url, head, res_body);
            return Err(());
        }
    }
}

async fn facebook_login<C, E>(client: &mut C, username: &str, password: &str, mut cookies_jar: Option<&mut HashMap<String, String>>) -> Result<(), ()>
where C: Service<Request<Body>, Response=Response<Body>, Error=E>,
    E: Display,
{
    let body = call(client, "GET", "https://m.facebook.com/", Some({
            let mut headers = HashMap::new();
            headers.insert("Referer", String::from("https://www.google.com/"));
            headers.insert("User-Agent", String::from("Nokia-MIT-Browser/3.0"));
            headers
        }), None, None).await?;

    let captures = FACEBOOK_LOGIN_FORM.captures(&body).ok_or_else(|| error!("Facebook login form not found"))?;
    let url = format!("https://m.facebook.com{}", captures.get(1).map(|m| m.as_str()).ok_or_else(|| error!("Facebook login form URL not found"))?);
    let form = captures.get(2).map(|m| m.as_str()).ok_or_else(|| error!("Facebook login form contents not found"))?;

    let mut fields = HashMap::new();
    INPUT_FIELDS.captures_iter(form).for_each(|m| if let Some(key) = m.get(1) {
        fields.insert(key.as_str(), m.get(3).map(|s| s.as_str()).unwrap_or_else(|| ""));
    });

    *(fields.get_mut("email").ok_or_else(|| error!("Facebook email field not found"))?) = username;
    *(fields.get_mut("pass").ok_or_else(|| error!("Facebook pass field not found"))?) = password;

    let req_body = fields.into_iter()
        .map(|(key, value)| format!("{}={}", utf8_percent_encode(key, NON_ALPHANUMERIC), utf8_percent_encode(value, NON_ALPHANUMERIC)))
        .collect::<Vec<String>>()
        .join("&");

    let mut temp_cookie_jar = HashMap::new();
    call(client, "POST", &url, Some({
            let mut headers = HashMap::new();
            headers.insert("Referer", String::from("https://m.facebook.com/"));
            headers.insert("User-Agent", String::from("Nokia-MIT-Browser/3.0"));
            headers.insert("Content-Type", String::from("application/x-www-form-urlencoded"));
            headers
        }), Some(req_body), Some(&mut temp_cookie_jar)).await?;
    temp_cookie_jar.get("c_user").ok_or_else(|| error!("Facebook login failed"))?;

    if let Some(ref mut cj) = cookies_jar {
        for (key, value) in temp_cookie_jar.into_iter() {
            cj.insert(key, value);
        }
    }

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
pub struct Intel<'a, C, E>
where C: Service<Request<Body>, Response=Response<Body>, Error=E>,
    E: Display, {
    username: &'a str,
    password: &'a str,
    client: &'a mut C,
    cookies_jar: HashMap<String, String>,
    api_version: Option<String>,
}

impl<'a, C, E> Intel<'a, C, E>
where C: Service<Request<Body>, Response=Response<Body>, Error=E>,
    E: Display, {
    /// creates a new Ingress Intel web client login
    pub fn new(client: &'a mut C, username: &'a str, password: &'a str) -> Self {
        Intel {
            username,
            password,
            client,
            cookies_jar: HashMap::new(),
            api_version: None,
        }
    }

    async fn login(&mut self) -> Result<(), ()> {
        if self.api_version.is_some() {
            return Ok(());
        }

        // login into facebook
        facebook_login(&mut self.client, &self.username, &self.password, Some(&mut self.cookies_jar)).await?;

        // retrieve facebook login url
        let intel = call(&mut self.client, "GET", "https://intel.ingress.com/", None, None, None).await?;
        let url = INTEL_URLS.captures_iter(&intel)
            .map(|m| m.get(1).map(|s| s.as_str()))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .find(|s| s.starts_with("https://www.facebook.com/"))
            .ok_or_else(|| error!("Can't retrieve Intel's Facebook login URL"))?;

        let intel = call(&mut self.client, "GET", url, Some({
                let mut headers = HashMap::new();
                headers.insert("Referer", String::from("https://intel.ingress.com/"));
                headers.insert("User-Agent", String::from("Nokia-MIT-Browser/3.0"));
                headers
            }), None, Some(&mut self.cookies_jar)).await?;
        let captures = API_VERSION.captures(&intel).ok_or_else(|| error!("Can't find Intel API version"))?;
        self.api_version = Some(captures.get(1).map(|m| m.as_str().to_owned()).ok_or_else(|| error!("Can't read Intel API version"))?);

        Ok(())
    }

    /// Retrieves entities informations for a given point
    pub async fn get_entities(&mut self, latitude: f64, longitude: f64) -> Result<entities::IntelResponse, ()> {
        self.login().await?;

        let body = json!({
            "tileKeys": get_tile_keys_around(latitude, longitude),
            "v": self.api_version.as_ref().unwrap(),
        });
        call_and_deserialize(&mut self.client, "POST", "https://intel.ingress.com/r/getEntities", Some({
                let mut headers = HashMap::new();
                headers.insert("Referer", String::from("https://intel.ingress.com/"));
                headers.insert("Origin", String::from("https://intel.ingress.com/"));
                headers.insert("Content-Type", String::from("application/json"));
                headers.insert("X-CSRFToken", self.cookies_jar["csrftoken"].clone());
				// headers.insert("User-Agent", String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36"));
                headers
            }), Some(body.to_string()), Some(&mut self.cookies_jar)).await
    }

    /// Retrieves informations for a given portal
    pub async fn get_portal_details(&mut self, portal_id: &str) -> Result<portal_details::IntelResponse, ()> {
        self.login().await?;

        let body = json!({
            "guid": portal_id,
            "v": self.api_version.as_ref().unwrap(),
        });
        call_and_deserialize(&mut self.client, "POST", "https://intel.ingress.com/r/getPortalDetails", Some({
                let mut headers = HashMap::new();
                headers.insert("Referer", String::from("https://intel.ingress.com/"));
                headers.insert("Origin", String::from("https://intel.ingress.com/"));
                headers.insert("Content-Type", String::from("application/json"));
                headers.insert("X-CSRFToken", self.cookies_jar["csrftoken"].clone());
				// headers.insert("User-Agent", String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36"));
                headers
            }), Some(body.to_string()), Some(&mut self.cookies_jar)).await
    }
}


#[cfg(test)]
mod tests {
    use super::Intel;

    use std::env;

    use hyper::{client::Client, Body};

    use hyper_tls::HttpsConnector;

    use once_cell::sync::Lazy;

    use log::info;

    static USERNAME: Lazy<String> = Lazy::new(|| env::var("USERNAME").expect("Missing USERNAME env var"));
    static PASSWORD: Lazy<String> = Lazy::new(|| env::var("PASSWORD").expect("Missing PASSWORD env var"));
    static LATITUDE: Lazy<Option<f64>> = Lazy::new(|| env::var("LATITUDE").map(|s| s.parse().expect("LATITUDE must be a float")).ok());
    static LONGITUDE: Lazy<Option<f64>> = Lazy::new(|| env::var("LONGITUDE").map(|s| s.parse().expect("LONGITUDE must be a float")).ok());
    static PORTAL_ID: Lazy<Option<String>> = Lazy::new(|| env::var("PORTAL_ID").ok());

    #[tokio::test]
    async fn login() -> Result<(), ()> {
        env_logger::try_init().ok();

        let https = HttpsConnector::new();
        let mut client = Client::builder().build::<_, Body>(https);

        let mut intel = Intel::new(&mut client, USERNAME.as_str(), PASSWORD.as_str());
        if let (Some(latitude), Some(longitude)) = (*LATITUDE, *LONGITUDE) {
            info!("get_entities {:?}", intel.get_entities(latitude, longitude).await?);
        }
        if let Some(portal_id) = &*PORTAL_ID {
            info!("get_portal_details {:?}", intel.get_portal_details(portal_id.as_str()).await?);
        }

        Ok(())
    }
}
