use std::{collections::HashMap, sync::Arc};

use reqwest::Method;
use serde_json::json;
use smol_str::SmolStr;
use tokio::sync::Mutex;
use tracing::error;

use crate::{Error, call, entities, get_cookies};

pub(crate) struct Params<'a> {
    pub(crate) inner: &'a super::Intel<'a>,
    pub(crate) tiles: Mutex<HashMap<SmolStr, TileState>>,
    pub(crate) api_version: SmolStr,
    pub(crate) csrftoken: SmolStr,
}

impl Params<'_> {
    pub(crate) async fn get_tiles(self: Arc<Self>) -> Option<Vec<entities::IntelEntities>> {
        let mut lock = self.tiles.lock().await;
        let ids = lock
            .iter_mut()
            .filter_map(|(id, status)| {
                status.is_free().then(|| {
                    *status = TileState::Busy;
                    id.clone()
                })
            })
            .take(25)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return None;
        }
        let body = json!({
            "tileKeys": ids,
            "v": self.api_version.clone(),
        });
        drop(lock);

        let inner_call = async {
            let req = self
                .inner
                .client
                .request(Method::POST, "https://intel.ingress.com/r/getEntities")
                .header("Referer", "https://intel.ingress.com/")
                .header("Origin", "https://intel.ingress.com/")
                .header("Cookie", get_cookies(&self.inner.cookie_store).await)
                .header("X-CSRFToken", self.csrftoken.as_str())
                .json(&body)
                .build()
                .map_err(|e| {
                    error!("error building entities request: {}", e);
                    Error::EntityRequest
                })?;

            call(&self.inner.client, req, &self.inner.cookie_store)
                .await?
                .json::<entities::IntelResponse>()
                .await
                .map_err(|e| {
                    error!("error deserializing entities response: {}", e);
                    Error::Deserialize
                })
        };

        if let Ok(res) = inner_call.await {
            let mut lock = self.tiles.lock().await;
            let mut ret = vec![];
            for (id, res) in res.result.map.into_iter() {
                if let entities::IntelResult::Entities(portals) = res {
                    ret.push(portals);
                    lock.insert(id, TileState::Done);
                } else {
                    lock.insert(id, TileState::Free);
                }
            }
            Some(ret)
        } else {
            let mut lock = self.tiles.lock().await;
            for id in ids {
                lock.insert(id, TileState::Free);
            }
            None
        }
    }

    pub(crate) async fn get_counts(self: Arc<Self>) -> (Arc<Self>, bool) {
        let lock = self.tiles.lock().await;
        let (free, busy, done) = lock.iter().fold((0, 0, 0), |(free, busy, done), (_, status)| match status {
            TileState::Free => (free + 1, busy, done),
            TileState::Busy => (free, busy + 1, done),
            TileState::Done => (free, busy, done + 1),
        });
        drop(lock);
        tracing::debug!("{free} free, {busy} busy, {done} done");
        (self, free + busy > 0)
    }
}

#[derive(Debug)]
pub(crate) enum TileState {
    Free,
    Busy,
    Done,
}

impl TileState {
    pub(crate) fn is_free(&self) -> bool {
        matches!(self, TileState::Free)
    }

    // pub(crate) fn is_busy(&self) -> bool {
    //     matches!(self, TileState::Busy)
    // }

    // pub(crate) fn is_done(&self) -> bool {
    //     matches!(self, TileState::Done)
    // }
}
