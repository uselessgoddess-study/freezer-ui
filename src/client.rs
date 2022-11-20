use crate::{
    model::{Freezer, Product},
    utils::Result,
};
use bytes::Bytes;
use json::json;

use reqwest::StatusCode;
use std::{
    fmt::{Debug, Formatter},
    ops::Deref,
    thread,
    time::Duration,
};

macro_rules! api {
    ($api:expr, $($tt:tt)*) => {
        format!("{}/{}", $api, format!($($tt)*))
    };
}

pub struct Client {
    api: String,
    inner: reqwest::Client,
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client {{ api: {}, client: reqwest::Client }}", self.api)
    }
}

impl Deref for Client {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        // todo: don't forget remove
        thread::sleep(Duration::from_secs(1));

        &self.inner
    }
}

impl Client {
    pub(crate) const DEFAULT_API: &'static str = "http://localhost:1228/api";

    pub fn new(api: &str, inner: reqwest::Client) -> Self {
        Self {
            api: api.to_owned(),
            inner,
        }
    }

    pub async fn login(&self, login: &str) -> Result<()> {
        Ok(self
            .get(api!(self.api, "auth"))
            .json(&json!(
                {
                    "login": login,
                }
            ))
            .send()
            .await?
            .bytes()
            .await
            .map(|_| ())?)
    }

    pub async fn image_bytes(&self, id: &str) -> Result<Bytes> {
        Ok(self
            .get(api!(self.api, "freezers/{id}/image"))
            .send()
            .await?
            .bytes()
            .await?)
    }

    pub async fn freezer(&self, id: &str) -> Result<Freezer> {
        Ok(self
            .get(api!(self.api, "freezers/{id}"))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn update_freezer(&self, freezer: Freezer) -> Result<Option<Freezer>> {
        let res = self
            .post(api!(self.api, "freezers/update"))
            .json(&freezer)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(Some(res.json().await?))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_freezer(&self, id: &str) -> Result<bool> {
        self.delete(api!(self.api, "freezers/{id}"))
            .send()
            .await
            .map(|res| res.status().is_success())
            .map_err(Into::into)
    }

    pub async fn freezers(&self) -> Result<Vec<String>> {
        Ok(self
            .get(api!(self.api, "freezers"))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn product(&self, id: &str) -> Result<Option<Product>> {
        match self.get(api!(self.api, "products/{id}")).send().await {
            Ok(res) => {
                if res.status() == StatusCode::NOT_FOUND {
                    Ok(None)
                } else {
                    Ok(Some(res.json().await?))
                }
            }
            #[allow(clippy::option_if_let_else)]
            Err(err) => match err.status() {
                // fixme: useless branch
                Some(status) if status == StatusCode::NOT_FOUND => Ok(None),
                _ => Err(err.into()),
            },
        }
    }

    pub async fn freezers_by(
        &self,
        limit: impl Into<Option<usize>> + Send + Copy,
        offset: impl Into<Option<usize>> + Send + Copy,
    ) -> Result<Vec<String>> {
        Ok(self
            .get(api!(self.api, "freezers"))
            .query(&json!(
                {
                    "limit": limit.into(),
                    "offset": offset.into(),
                }
            ))
            .send()
            .await?
            .json()
            .await?)
    }
}
