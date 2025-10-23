#![deny(warnings)]
#![deny(missing_docs)]

//! # ingress_intel_rs
//!
//! Ingress Intel API interface in pure Rust

use std::{borrow::Cow, collections::HashMap, convert::identity, iter::repeat, sync::Arc, time::Duration};

use once_cell::sync::{Lazy, OnceCell};
use percent_encoding::percent_decode_str;
use regex::Regex;
use reqwest::{Client, Method, Request, Response};
use serde_json::{json, value::Value};
use smol_str::{SmolStr, ToSmolStr};
use tokio::sync::{Mutex, RwLock};
use tokio_stream::{Stream, StreamExt};
use tracing::error;

mod get_entities_in_range;
mod tile_key;
mod utils;
use tile_key::TileKey;

/// getEntities endpoint resource
pub mod entities;

/// getPortalDetails endpoint resources
pub mod portal_details;

/// getPlexts endpoint resources
pub mod plexts;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0";

static INTEL_URLS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<a[^>]+href="([^"]+)""#).unwrap());
static FACEBOOK_LOGIN_FORM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<form[^>]+data-testid="royal_login_form"[^>]+action="([^"]+?)"[^>]+>([\s\S]+?)</form>"#).unwrap()
});
static INPUT_FIELDS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<input([^>]+)>"#).unwrap());
static INPUT_ATTRIBUTES: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([^\s="]+)="([^"]+)""#).unwrap());
// static COOKIE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([^=]+)=([^;]+)"#).unwrap());
static API_VERSION: Lazy<Regex> = Lazy::new(|| Regex::new(r"/jsc/gen_dashboard_(\w+)\.js").unwrap());

/// Error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Transport error
    #[error("Transport")]
    Transport,
    /// Status error
    #[error("Status")]
    Status,
    /// MissingFacebookUsername error
    #[error("MissingFacebookUsername")]
    MissingFacebookUsername,
    /// MissingFacebookPassword error
    #[error("MissingFacebookPassword")]
    MissingFacebookPassword,
    /// FacebookUrl error
    #[error("FacebookUrl")]
    FacebookUrl,
    /// FirstFacebookRequest error
    #[error("FirstFacebookRequest")]
    FirstFacebookRequest,
    /// FirstFacebookResponse error
    #[error("FirstFacebookResponse")]
    FirstFacebookResponse,
    /// SecondFacebookRequest error
    #[error("SecondFacebookRequest")]
    SecondFacebookRequest,
    /// LoginForm error
    #[error("LoginForm")]
    LoginForm,
    /// LoginFailed error
    #[error("LoginFailed")]
    LoginFailed,
    /// FirstIntelRequest error
    #[error("FirstIntelRequest")]
    FirstIntelRequest,
    /// SecondIntelRequest error
    #[error("SecondIntelRequest")]
    SecondIntelRequest,
    /// CsrfToken error
    #[error("CsrfToken")]
    CsrfToken,
    /// IntelApiVersion error
    #[error("IntelApiVersion")]
    IntelApiVersion,
    /// EntityRequest error
    #[error("EntityRequest")]
    EntityRequest,
    /// PortalDetailsRequest error
    #[error("PortalDetailsRequest")]
    PortalDetailsRequest,
    /// PlextsRequest error
    #[error("PlextsRequest")]
    PlextsRequest,
    /// Deserialize error
    #[error("Deserialize")]
    Deserialize,
    /// Join error
    #[error("Join")]
    Join,
}

async fn call(
    client: &Client,
    req: Request,
    cookie_store: &RwLock<HashMap<SmolStr, SmolStr>>,
) -> Result<Response, Error> {
    let url = req.url().to_smolstr();
    let res = client
        .execute(req)
        .await
        .map_err(|e| {
            error!("error receiving response from {}: {}", url, e);
            Error::Transport
        })?
        .error_for_status()
        .map_err(|e| {
            error!("unsucessfull response from {}: {}", url, e);
            Error::Status
        })?;

    let mut lock = cookie_store.write().await;
    res.cookies().for_each(|c| {
        lock.insert(c.name().to_smolstr(), c.value().to_smolstr());
    });

    Ok(res)
}

async fn get_cookies(cookie_store: &RwLock<HashMap<SmolStr, SmolStr>>) -> String {
    let lock = cookie_store.read().await;
    lock.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("; ")
}

async fn facebook_login(
    client: &Client,
    username: &str,
    password: &str,
    cookie_store: &RwLock<HashMap<SmolStr, SmolStr>>,
) -> Result<(), Error> {
    let req = client
        .request(Method::GET, "https://www.facebook.com/?_fb_noscript=1")
        // .header("Referer", "https://www.google.com/")
        .header("User-Agent", USER_AGENT)
        .build()
        .map_err(|e| {
            error!("error building first facebook request: {}", e);
            Error::FirstFacebookRequest
        })?;

    let body = call(client, req, cookie_store).await?.text().await.map_err(|e| {
        error!("error encoding response text: {}", e);
        Error::FirstFacebookResponse
    })?;

    let captures = FACEBOOK_LOGIN_FORM.captures(&body).ok_or_else(|| {
        error!("Facebook login form not found");
        Error::LoginForm
    })?;
    let url = format!(
        "https://www.facebook.com{}",
        captures
            .get(1)
            .and_then(|m| percent_decode_str(&m.as_str().replace("&amp;", "&"))
                .decode_utf8()
                .ok()
                .map(|s| s.to_smolstr()))
            .ok_or_else(|| {
                error!("Facebook login form URL not found\nbody: {}", body);
                Error::LoginForm
            })?
    );
    let form = captures.get(2).map(|m| m.as_str()).ok_or_else(|| {
        error!("Facebook login form contents not found");
        Error::LoginForm
    })?;

    let mut fields = Value::Null;
    for m in INPUT_FIELDS.captures_iter(form) {
        if let Some(input) = m.get(1) {
            let (name, value) =
                INPUT_ATTRIBUTES.captures_iter(input.as_str()).fold((None, None), |(mut name, mut value), im| {
                    let key = im.get(1).map(|s| s.as_str());
                    if key == Some("name") {
                        name = im.get(2).map(|s| s.as_str());
                    } else if key == Some("value") {
                        value = im.get(2).map(|s| s.as_str());
                    }
                    (name, value)
                });
            if let Some(key) = name {
                // if key != "_fb_noscript" && key != "sign_up" {
                fields[key] = Value::from(value.unwrap_or_default());
                // }
            }
        }
    }

    fields["email"] = Value::from(username);
    fields["pass"] = Value::from(password);

    let req = client
        .request(Method::POST, &url)
        // .header("Referer", "https://www.facebook.com/")
        // .header("Origin", "https://www.facebook.com/")
        .header("User-Agent", USER_AGENT)
        .header("Cookie", get_cookies(cookie_store).await)
        .form(&fields)
        .build()
        .map_err(|e| {
            error!("error building second facebook request: {}", e);
            Error::SecondFacebookRequest
        })?;

    let res = call(client, req, cookie_store).await?;
    res.cookies().find(|c| c.name() == "c_user").ok_or_else(|| {
        error!("Facebook login failed");
        Error::LoginFailed
    })?;

    Ok(())
}

fn get_tile_keys_around(
    latitude: f64,
    longitude: f64,
    zoom: Option<u8>,
    min_level: Option<u8>,
    max_level: Option<u8>,
    health: Option<u8>,
) -> Vec<SmolStr> {
    let base = TileKey::new(latitude, longitude, zoom, min_level, max_level, health);

    vec![
        base.to_smolstr(),
        (base + (-1, -1)).to_smolstr(),
        (base + (-1, 0)).to_smolstr(),
        (base + (-1, 1)).to_smolstr(),
        (base + (0, -1)).to_smolstr(),
        (base + (0, 1)).to_smolstr(),
        (base + (1, 0)).to_smolstr(),
        (base + (1, 1)).to_smolstr(),
        (base + (1, -1)).to_smolstr(),
    ]
}

/// Represents an Ingress Intel web client login
pub struct Intel<'a> {
    username: Option<Cow<'a, str>>,
    password: Option<Cow<'a, str>>,
    client: Cow<'a, Client>,
    cookie_store: RwLock<HashMap<SmolStr, SmolStr>>,
    api_version: OnceCell<SmolStr>,
    csrftoken: OnceCell<SmolStr>,
}

impl<'a> Intel<'a> {
    /// creates a new Ingress Intel web client login from existing Client
    pub fn new(client: &'a Client, username: Option<Cow<'a, str>>, password: Option<Cow<'a, str>>) -> Self {
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
    pub fn build(username: Option<Cow<'a, str>>, password: Option<Cow<'a, str>>) -> Self {
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
        N: ToSmolStr,
        V: ToSmolStr,
    {
        let mut lock = self.cookie_store.write().await;
        lock.insert(name.to_smolstr(), value.to_smolstr());
    }

    /// adds multiple cookies to the store
    pub async fn add_cookies<I, N, V>(&self, iter: I)
    where
        I: IntoIterator<Item = (N, V)>,
        N: ToSmolStr,
        V: ToSmolStr,
    {
        let mut lock = self.cookie_store.write().await;
        for (name, value) in iter {
            lock.insert(name.to_smolstr(), value.to_smolstr());
        }
    }

    async fn cookie_exists(&self, cookie: &str) -> bool {
        let lock = self.cookie_store.read().await;
        lock.get(cookie).is_some()
    }

    /// performs login, if necessary
    pub async fn login(&self) -> Result<(), Error> {
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
                    self.username.as_ref().ok_or_else(|| {
                        error!("Missing facebok username");
                        Error::MissingFacebookUsername
                    })?,
                    self.password.as_ref().ok_or_else(|| {
                        error!("Missing facebook password");
                        Error::MissingFacebookPassword
                    })?,
                    &self.cookie_store,
                )
                .await?;
            }

            // retrieve facebook login url
            let req = self.client.request(Method::GET, "https://intel.ingress.com/").build().map_err(|e| {
                error!("error building first intel request: {}", e);
                Error::FirstIntelRequest
            })?;
            let intel = call(&self.client, req, &self.cookie_store).await?.text().await.map_err(|e| {
                error!("error encoding first intel response: {}", e);
                Error::FirstIntelRequest
            })?;
            INTEL_URLS
                .captures_iter(&intel)
                .flat_map(|m| m.get(1).map(|s| s.as_str()))
                .find(|s| s.starts_with("https://www.facebook.com/"))
                .ok_or_else(|| {
                    error!("Can't retrieve Intel's Facebook login URL");
                    Error::FacebookUrl
                })?
                .to_smolstr()
        } else {
            SmolStr::from("https://intel.ingress.com/")
        };

        let req = self
            .client
            .request(Method::GET, url.as_str())
            .header("User-Agent", USER_AGENT)
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .build()
            .map_err(|e| {
                error!("error building second intel request: {}", e);
                Error::SecondIntelRequest
            })?;
        let res = call(&self.client, req, &self.cookie_store).await?;
        let csrftoken =
            res.cookies().find(|c| c.name() == "csrftoken").map(|c| c.value().to_smolstr()).ok_or_else(|| {
                error!("Can't find csrftoken Cookie");
                Error::CsrfToken
            })?;
        self.csrftoken.set(csrftoken).map_err(|_| {
            error!("Can't set csrftoken");
            Error::CsrfToken
        })?;
        let intel = res.text().await.map_err(|e| {
            error!("error encoding second intel response: {}", e);
            Error::SecondIntelRequest
        })?;

        let captures = API_VERSION.captures(&intel).ok_or_else(|| {
            error!("Can't find Intel API version");
            Error::IntelApiVersion
        })?;
        let api_version = captures.get(1).map(|m| m.as_str().to_smolstr()).ok_or_else(|| {
            error!("Can't read Intel API version");
            Error::IntelApiVersion
        })?;
        self.api_version.set(api_version).map_err(|_| {
            error!("Can't set api_version");
            Error::IntelApiVersion
        })?;

        Ok(())
    }

    /// Retrieves entities informations for a given point
    pub async fn get_entities_around(
        &self,
        latitude: f64,
        longitude: f64,
        zoom: Option<u8>,
        min_level: Option<u8>,
        max_level: Option<u8>,
        health: Option<u8>,
    ) -> Result<entities::IntelResponse, Error> {
        self.login().await?;

        let csrftoken = self.csrftoken.get().ok_or_else(|| {
            error!("missing CSRFToken");
            Error::CsrfToken
        })?;

        let body = json!({
            "tileKeys": get_tile_keys_around(latitude, longitude, zoom, min_level, max_level, health),
            "v": self.api_version.get().ok_or_else(|| {
                error!("missing API version");
                Error::IntelApiVersion
            })?,
        });

        let req = self
            .client
            .request(Method::POST, "https://intel.ingress.com/r/getEntities")
            .header("Referer", "https://intel.ingress.com/")
            .header("Origin", "https://intel.ingress.com/")
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .header("X-CSRFToken", csrftoken.as_str())
            .json(&body)
            .build()
            .map_err(|e| {
                error!("error building entities request: {}", e);
                Error::EntityRequest
            })?;

        call(&self.client, req, &self.cookie_store).await?.json().await.map_err(|e| {
            error!("error deserializing entities response: {}", e);
            Error::Deserialize
        })
    }

    /// Retrieves entities informations for a given point
    #[allow(clippy::too_many_arguments)]
    pub async fn get_entities_in_range(
        &'a self,
        from: (f64, f64),
        to: (f64, f64),
        zoom: Option<u8>,
        min_level: Option<u8>,
        max_level: Option<u8>,
        health: Option<u8>,
        throttle: Duration,
    ) -> Result<impl Stream<Item = Vec<entities::IntelEntities>> + Send + Sync + 'a, Error> {
        self.login().await?;

        let api_version = self.api_version.get().map(ToOwned::to_owned).ok_or_else(|| {
            error!("missing API version");
            Error::IntelApiVersion
        })?;
        let csrftoken = self.csrftoken.get().map(ToOwned::to_owned).ok_or_else(|| {
            error!("missing CSRFToken");
            Error::CsrfToken
        })?;

        let tile_keys = TileKey::range(from, to, zoom, min_level, max_level, health);

        let params = get_entities_in_range::Params {
            inner: self,
            tiles: Mutex::new(
                tile_keys.map(|tile| (tile, get_entities_in_range::TileState::Free)).collect::<HashMap<_, _>>(),
            ),
            api_version,
            csrftoken,
        };

        // situation here is quite catastophic, every call can fail on the outer level, aka the call itself fails,
        // but also on the inner level, aka the single tile key has an error
        // at this point we need to make everything retriable

        Ok(tokio_stream::iter(repeat(Arc::new(params)))
            .throttle(throttle)
            .then(get_entities_in_range::Params::get_counts)
            .take_while(|(_, counts)| *counts)
            .map(|(params, _)| params)
            .then(get_entities_in_range::Params::get_tiles)
            .filter_map(identity))
    }

    /// Retrieves informations for a given portal
    pub async fn get_portal_details(&self, portal_id: &str) -> Result<portal_details::IntelResponse, Error> {
        self.login().await?;

        let csrftoken = self.csrftoken.get().ok_or_else(|| {
            error!("missing CSRFToken");
            Error::CsrfToken
        })?;

        let body = json!({
            "guid": portal_id,
            "v": self.api_version.get().unwrap(),
        });

        let req = self
            .client
            .request(Method::POST, "https://intel.ingress.com/r/getPortalDetails")
            .header("Referer", "https://intel.ingress.com/")
            .header("Origin", "https://intel.ingress.com/")
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .header("X-CSRFToken", csrftoken.as_str())
            .json(&body)
            .build()
            .map_err(|e| {
                error!("error building portal details request: {}", e);
                Error::PortalDetailsRequest
            })?;

        call(&self.client, req, &self.cookie_store).await?.json().await.map_err(|e| {
            error!("error deserializing portal details response: {}", e);
            Error::Deserialize
        })
    }

    /// Retrieves COMM contents
    pub async fn get_plexts(
        &self,
        from: [u64; 2],
        to: [u64; 2],
        tab: plexts::Tab,
        min_timestamp_ms: Option<i64>,
        max_timestamp_ms: Option<i64>,
    ) -> Result<plexts::IntelResponse, Error> {
        self.login().await?;

        let csrftoken = self.csrftoken.get().ok_or_else(|| {
            error!("missing CSRFToken");
            Error::CsrfToken
        })?;

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

        let req = self
            .client
            .request(Method::POST, "https://intel.ingress.com/r/getPlexts")
            .header("Referer", "https://intel.ingress.com/")
            .header("Origin", "https://intel.ingress.com/")
            .header("Cookie", get_cookies(&self.cookie_store).await)
            .header("X-CSRFToken", csrftoken.as_str())
            .json(&body)
            .build()
            .map_err(|e| {
                error!("error building portal details request: {}", e);
                Error::PlextsRequest
            })?;

        call(&self.client, req, &self.cookie_store).await?.json().await.map_err(|e| {
            error!("error deserializing portal details response: {}", e);
            Error::Deserialize
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, env, time::Duration};

    use tokio_stream::StreamExt;
    use tracing::info;

    async fn login() -> super::Intel<'static> {
        tracing_subscriber::fmt::try_init().ok();

        let intel =
            super::Intel::build(env::var("USERNAME").ok().map(Cow::Owned), env::var("PASSWORD").ok().map(Cow::Owned));

        if let Ok(cookies) = env::var("COOKIES") {
            intel
                .add_cookies(cookies.split("; ").filter_map(|cookie| {
                    let (pos, _) = cookie.match_indices('=').next()?;
                    Some((&cookie[0..pos], &cookie[(pos + 1)..]))
                }))
                .await;
        }

        intel
    }

    #[test_with::env(LATITUDE, LONGITUDE)]
    #[tokio::test]
    async fn get_entities_around() {
        let intel = login().await;
        info!(
            "get_entities_around {:#?}",
            intel
                .get_entities_around(
                    env::var("LATITUDE").unwrap().parse().unwrap(),
                    env::var("LONGITUDE").unwrap().parse().unwrap(),
                    env::var("ZOOM").ok().as_deref().map(str::parse).transpose().unwrap(),
                    env::var("MIN_LEVEL").ok().as_deref().map(str::parse).transpose().unwrap(),
                    None,
                    None
                )
                .await
                .unwrap()
        );
    }

    #[test_with::env(LATITUDE_FROM, LONGITUDE_FROM, LATITUDE_TO, LONGITUDE_TO)]
    #[tokio::test]
    async fn get_entities_in_range() {
        let intel = login().await;
        info!(
            "get_entities_in_range {:#?}",
            intel
                .get_entities_in_range(
                    (
                        env::var("LATITUDE_FROM").unwrap().parse().unwrap(),
                        env::var("LONGITUDE_FROM").unwrap().parse().unwrap()
                    ),
                    (
                        env::var("LATITUDE_TO").unwrap().parse().unwrap(),
                        env::var("LONGITUDE_TO").unwrap().parse().unwrap()
                    ),
                    env::var("ZOOM").ok().as_deref().map(str::parse).transpose().unwrap(),
                    env::var("MIN_LEVEL").ok().as_deref().map(str::parse).transpose().unwrap(),
                    None,
                    None,
                    Duration::from_millis(1500), // 40 in 60 seconds
                )
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
        );
    }

    #[test_with::env(PORTAL_ID)]
    #[tokio::test]
    async fn get_portal_details() {
        let intel = login().await;
        info!(
            "get_portal_details {:?}",
            intel.get_portal_details(env::var("PORTAL_ID").unwrap().as_str()).await.unwrap()
        );
    }
}
