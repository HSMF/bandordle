//! # Last.fm API bindings
//!
//!
//! ## Usage Example
//!
//! ```
//! let client = Client::new(shared_secret, api_key);
//! ```

use std::fmt::Write;

use md5::{Digest, Md5};
use serde::de::DeserializeOwned;

use crate::api_types::{TopAlbums, TopArtists, TopTracks};

pub mod api_types;

macro_rules! request_builder {
    (
        $(#[doc = $doc:literal])?
        struct $name:ident<$life:lifetime> {
        method: $method:expr,
        required: { $(
            $required:ident: $rtyp:ty
        ),* $(,)? } $(,)?
        optional: { $(
            $( #[doc = $odoc:literal] )?
            $optional:ident: $otyp:ty
            ),* $(,)? } $(,)?
        }
        => $api:ty
        => $ret:ty ) => {
        $(#[doc = $doc])?
        pub struct $name<$life> {
            client: &$life Client,
            $($required: $rtyp,)*
            $($optional: Option<$otyp>,)*
        }

        impl<$life> $name<$life> {
            fn new(client: &$life Client,  $($required: $rtyp),*) -> Self {
                Self {
                    client,
                    $($required,)*
                    $($optional: None),*
                }
            }

            $(
                $( #[doc = $odoc] )?
                pub fn $optional(mut self, $optional: $otyp ) -> Self {
                    self.$optional = Some($optional);
                    self
                }
            )*

            pub async fn send(self) -> Result<$ret, Error> {
                #[allow(unused_mut)]
                let mut args: Vec<(&str, String)> = vec![
                    $((stringify!($required), self.$required.to_string() ) )*
                ];

                $(
                    if let Some($optional) = self.$optional {
                        args.push((stringify!($optional), $optional.to_string()));
                    }
                )*

                self.client
                    .make_request::<$api>(
                        $method,
                        args.iter().map(|x| (x.0, x.1.as_str())),
                    )
                    .await?
                    .into_result()
                    .map_err(Error::Api)
            }

        }
    };
}

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
    base_url: String,
}

impl Client {
    pub fn new(shared_secret: String, api_key: String) -> Self {
        Self {
            shared_secret,
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://ws.audioscrobbler.com/2.0/".into(),
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
            .get(&self.base_url)
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

    pub fn top_tracks<'a>(&'a self, user: &'a str) -> GetTopTracks<'a> {
        GetTopTracks::new(self, user)
    }

    pub fn top_albums<'a>(&'a self, user: &'a str) -> GetTopAlbums<'a> {
        GetTopAlbums::new(self, user)
    }

    pub fn top_artists<'a>(&'a self, user: &'a str) -> GetTopArtists<'a> {
        GetTopArtists::new(self, user)
    }

    pub fn top_artists_charts<'a>(&'a self) -> GetTopArtistsCharts<'a> {
        GetTopArtistsCharts::new(self)
    }

    pub fn top_tags_charts<'a>(&'a self) -> GetTopTagsCharts<'a> {
        GetTopTagsCharts::new(self)
    }

    pub fn top_tracks_charts<'a>(&'a self) -> GetTopTracksCharts<'a> {
        GetTopTracksCharts::new(self)
    }
}

request_builder! {
    struct GetTopAlbums<'a> {
        method: "user.getTopAlbums",
        required: {
            user: &'a str,
        }
        optional: {
            /// The time period over which to retrieve top artists for.
            period: api_types::Period,
            /// The page number to fetch. Defaults to first page.
            page: usize,
            /// The number of results to fetch per page. Defaults to 50.
            limit: usize,
        }
    }
    => api_types::GetTopAlbumsResponse
    => TopAlbums
}

request_builder! {
    struct GetTopTracks<'a> {
        method: "user.getTopTracks",
        required: {
            user: &'a str,
        }
        optional: {
            /// The time period over which to retrieve top tracks for.
            period: api_types::Period,
            /// The page number to fetch. Defaults to first page.
            page: usize,
            /// The number of results to fetch per page. Defaults to 50.
            limit: usize,
        }
    }
    => api_types::GetTopTracksResponse
    => TopTracks
}

request_builder! {
    struct GetTopArtists<'a> {
        method: "user.getTopArtists",
        required: {
            user: &'a str,
        }
        optional: {
            /// The time period over which to retrieve top artists for.
            period: api_types::Period,
            /// The page number to fetch. Defaults to first page.
            page: usize,
            /// The number of results to fetch per page. Defaults to 50.
            limit: usize,
        }
    }
    => api_types::GetTopArtistsResponse
    => TopArtists
}

request_builder! {
    struct GetTopArtistsCharts<'a> {
        method: "chart.getTopArtists",
        required: { }
        optional: {
            /// The page number to fetch. Defaults to first page.
            page: usize,
            /// The number of results to fetch per page. Defaults to 50.
            limit: usize,
        }
    }
    => api_types::chart::GetTopArtistsResponse
    => api_types::chart::TopArtists
}

request_builder! {
    struct GetTopTagsCharts<'a> {
        method: "chart.getTopTags",
        required: { }
        optional: {
            /// The page number to fetch. Defaults to first page.
            page: usize,
            /// The number of results to fetch per page. Defaults to 50.
            limit: usize,
        }
    }
    => api_types::chart::GetTopTagsResponse
    => api_types::chart::TopTags
}

request_builder! {
    struct GetTopTracksCharts<'a> {
        method: "chart.getTopTracks",
        required: { }
        optional: {
            /// The page number to fetch. Defaults to first page.
            page: usize,
            /// The number of results to fetch per page. Defaults to 50.
            limit: usize,
        }
    }
    => api_types::chart::GetTopTracksResponse
    => api_types::chart::TopTracks
}
