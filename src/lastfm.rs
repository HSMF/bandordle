use std::fmt::Write;

use md5::{Digest, Md5};
use serde::de::DeserializeOwned;

use crate::lastfm::api_types::TopAlbums;

pub mod api_types;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP {0}")]
    Http(reqwest::Error),
    #[error("Decoding {0}")]
    Decoding(quick_xml::DeError),
    #[error("Lastfm {0}")]
    Api(api_types::Error),
}

pub struct Client {
    shared_secret: String,
    api_key: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(shared_secret: String, api_key: String) -> Self {
        Self {
            shared_secret,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn make_request<'a, T>(
        &self,
        method: &str,
        args: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut args: Vec<_> = args.into_iter().collect();
        args.push(("method", method));
        args.push(("api_key", &self.api_key));
        args.sort_unstable();
        let mut hasher = Md5::new();
        for (k, v) in &args {
            hasher.update(k);
            hasher.update(v);
        }
        hasher.update(&self.shared_secret);
        let sign = hasher.finalize();
        let mut signature = String::with_capacity(2 * sign.len());
        for ch in sign {
            let _ = write!(&mut signature, "{ch:02x}");
        }

        use Error::{Decoding, Http};

        let resp = self
            .client
            .get("https://ws.audioscrobbler.com/2.0/")
            .query(&args)
            .query(&[("api_sig", signature)])
            .send()
            .await
            .map_err(Http)?
            .text()
            .await
            .map_err(Http)?;

        quick_xml::de::from_str(&resp).map_err(Decoding)
    }

    pub async fn authenticate(&self, token: &str) -> Result<api_types::Session, Error> {
        self.make_request::<api_types::AuthGetSessionResponse>(
            "auth.getSession",
            [("token", token)],
        )
        .await?
        .0
        .into_result()
        .map_err(Error::Api)
    }

    pub async fn get_top_albums(&self, user: &str) -> Result<TopAlbums, Error> {
        self.make_request::<api_types::GetTopAlbumsResponse>("user.getTopAlbums", [("user", user)])
            .await?
            .into_result()
            .map_err(Error::Api)
    }
}
