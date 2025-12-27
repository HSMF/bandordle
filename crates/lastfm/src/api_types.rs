use std::fmt::Display;

use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename = "error")]
pub struct Error {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "$value")]
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self.code.as_str() {
            "2" => "Invalid service - This service does not exist",
            "3" => "Invalid Method - No method with that name in this package",
            "4" => "Authentication Failed - You do not have permissions to access the service",
            "5" => "Invalid format - This service doesn't exist in that format",
            "6" => "Invalid parameters - Your request is missing a required parameter",
            "7" => "Invalid resource specified",
            "8" => "Operation failed - Something else went wrong",
            "9" => "Invalid session key - Please re-authenticate",
            "10" => "Invalid API key - You must be granted a valid key by last.fm",
            "11" => "Service Offline - This service is temporarily offline. Try again later.",
            "13" => "Invalid method signature supplied",
            "14" => "This token has not been authorized",
            "15" => "This token has expired",
            "16" => "There was a temporary error processing your request. Please try again",
            "26" => {
                "Suspended API key - Access for your account has been suspended, please contact Last.fm"
            }
            "29" => "Rate limit exceeded - Your IP has made too many requests in a short period",
            code => {
                write!(f, "unknown code ")?;
                code
            }
        };
        write!(f, "{}: {code}: {}", self.code, self.message)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "@status", content = "$value")]
#[serde(rename = "lfm")]
#[serde(rename_all = "kebab-case")]
pub enum LfmStatus<T> {
    Ok(T),
    Failed(Error),
}
impl<T> LfmStatus<T> {
    pub fn into_result(self) -> Result<T, Error> {
        match self {
            LfmStatus::Ok(x) => Ok(x),
            LfmStatus::Failed(error) => Err(error),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Session {
    pub name: String,
    pub key: String,
    pub subscriber: i32,
}

/// # Sample
/// ```xml
/// <lfm status="ok">
///   <session>
///     <name>MyLastFMUsername</name>
///     <key>d580d57f32848f5dcf574d1ce18d78b2</key>
///      <subscriber>0</subscriber>
///   </session>
/// </lfm>
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct AuthGetSessionResponse(pub LfmStatus<Session>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Period {
    #[serde(rename = "overall")]
    Overall,
    #[serde(rename = "7day")]
    SevenDay,
    #[serde(rename = "1month")]
    OneMonth,
    #[serde(rename = "3month")]
    ThreeMonth,
    #[serde(rename = "6month")]
    SixMonth,
    #[serde(rename = "12month")]
    TwelveMonth,
}
impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Period::Overall => "overall",
            Period::SevenDay => "7day",
            Period::OneMonth => "1month",
            Period::ThreeMonth => "3month",
            Period::SixMonth => "6month",
            Period::TwelveMonth => "12month",
        };
        write!(f, "{s}")
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "artist")]
pub struct ShortArtist {
    name: String,
    mbid: String,
    url: Url,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageSize {
    Small,
    Medium,
    Large,
    Extralarge,
    Mega,
    Unknown(String),
}

impl<'de> Deserialize<'de> for ImageSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        use ImageSize::{Extralarge, Large, Medium, Mega, Small};
        Ok(match s.as_str() {
            "small" => Small,
            "medium" => Medium,
            "large" => Large,
            "mega" => Mega,
            "extralarge" => Extralarge,
            _ => Self::Unknown(s),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "image")]
pub struct Image {
    #[serde(rename = "@size")]
    size: ImageSize,
    #[serde(rename = "$value")]
    url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "album")]
pub struct Album {
    #[serde(rename = "@rank")]
    pub rank: i64,
    pub name: String,
    pub playcount: i64,
    pub mbid: String,
    pub url: Url,
    pub artist: ShortArtist,
    #[serde(rename = "$value")]
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "artist")]
pub struct Artist {
    #[serde(rename = "@rank")]
    pub rank: i64,
    pub name: String,
    pub playcount: i64,
    pub mbid: String,
    pub url: Url,
    pub streamable: bool,
    #[serde(rename = "$value")]
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "artist")]
pub struct Track {
    #[serde(rename = "@rank")]
    pub rank: i64,
    pub name: String,
    pub playcount: i64,
    pub mbid: String,
    pub url: Url,
    pub streamable: bool,
    pub artist: ShortArtist,
    #[serde(rename = "$value")]
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "topalbums")]
pub struct TopAlbums {
    #[serde(rename = "@user")]
    pub user: String,
    // #[serde(rename = "@type")]
    // typ: Period,
    #[serde(rename = "$value")]
    pub albums: Vec<Album>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "topartists")]
pub struct TopArtists {
    #[serde(rename = "@user")]
    pub user: String,

    #[serde(rename = "$value")]
    pub artists: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "toptracks")]
pub struct TopTracks {
    #[serde(rename = "@user")]
    pub user: String,

    #[serde(rename = "$value")]
    pub artists: Vec<Track>,
}

pub mod chart {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "wiki")]
    pub struct Wiki {
        published: String,
        summary: String,
        content: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "artists")]
    pub struct Artist {
        name: String,
        playcount: i64,
        listeners: i64,
        mbid: String,
        url: Url,
        streamable: bool,
        #[serde(rename = "$value")]
        images: Vec<Image>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "tag")]
    pub struct Tag {
        name: String,
        url: Url,
        reach: i64,
        taggings: i64,
        streamable: bool,
        wiki: Wiki,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "track")]
    pub struct Track {
        name: String,
        playcount: i64,
        listeners: i64,
        mbid: Option<String>,
        url: Url,
        streamable: bool,
        artist: ShortArtist,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "artists")]
    pub struct TopArtists {
        #[serde(rename = "@page")]
        page: usize,
        #[serde(rename = "@perPage")]
        per_page: usize,
        #[serde(rename = "@totalPages")]
        total_pages: usize,
        #[serde(rename = "@total")]
        total: usize,
        #[serde(rename = "$value")]
        artists: Vec<Artist>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "tags")]
    pub struct TopTags {
        #[serde(rename = "@page")]
        page: usize,
        #[serde(rename = "@perPage")]
        per_page: usize,
        #[serde(rename = "@totalPages")]
        total_pages: usize,
        #[serde(rename = "@total")]
        total: usize,

        tags: Vec<Tag>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[serde(rename = "tags")]
    pub struct TopTracks {
        #[serde(rename = "@page")]
        page: usize,
        #[serde(rename = "@perPage")]
        per_page: usize,
        #[serde(rename = "@totalPages")]
        total_pages: usize,
        #[serde(rename = "@total")]
        total: usize,

        tags: Vec<Track>,
    }

    pub type GetTopArtistsResponse = LfmStatus<TopArtists>;

    pub type GetTopTagsResponse = LfmStatus<TopTags>;

    pub type GetTopTracksResponse = LfmStatus<TopTracks>;
}

pub type GetTopAlbumsResponse = LfmStatus<TopAlbums>;

pub type GetTopArtistsResponse = LfmStatus<TopArtists>;

pub type GetTopTracksResponse = LfmStatus<TopTracks>;

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    use quick_xml::de::from_str;

    fn i(size: ImageSize, url: &str) -> Image {
        Image {
            size,
            url: url.into(),
        }
    }

    #[test]
    fn status_failed() {
        let x: LfmStatus<()> = from_str(
            r#"
            <lfm status="failed">
                <error code="10">Invalid API Key</error>
            </lfm>
            "#,
        )
        .expect("can parse xml");
        assert_eq!(
            x,
            LfmStatus::Failed(Error {
                code: "10".into(),
                message: "Invalid API Key".into()
            })
        );
    }

    #[test]
    fn auth_get_session_response() {
        let x: AuthGetSessionResponse = from_str(
            r#"<lfm status="ok">
                <session>
                <name>MyLastFMUsername</name>
                <key>d580d57f32848f5dcf574d1ce18d78b2</key>
                <subscriber>0</subscriber>
                </session>
            </lfm>"#,
        )
        .expect("can parse");
        assert_eq!(
            x,
            AuthGetSessionResponse(LfmStatus::Ok(Session {
                name: "MyLastFMUsername".into(),
                key: "d580d57f32848f5dcf574d1ce18d78b2".into(),
                subscriber: 0
            }))
        );
    }

    #[test]
    fn user_get_top_albums_response() {
        let x: TopAlbums = from_str(
            r#"<topalbums user="RJ" type="overall">
<album rank="1">
  <name>Images and Words</name>
  <playcount>174</playcount>
  <mbid>f20971f2-c8ad-4d26-91ab-730f6dedafb2</mbid>  
  <url>
    http://www.last.fm/music/Dream+Theater/Images+and+Words
  </url>
  <artist>
    <name>Dream Theater</name>
    <mbid>28503ab7-8bf2-4666-a7bd-2644bfc7cb1d</mbid>
    <url>http://www.last.fm/music/Dream+Theater</url>
  </artist>
  <image size="small">...</image>
  <image size="medium">...</image>
  <image size="large">...</image>
</album>
</topalbums>"#,
        )
        .expect("can parse");
        assert_eq!(
            x,
            TopAlbums {
                user: "RJ".into(),
                albums: vec![Album {
                    rank: 1,
                    name: "Images and Words".into(),
                    playcount: 174,
                    mbid: "f20971f2-c8ad-4d26-91ab-730f6dedafb2".into(),
                    url: "http://www.last.fm/music/Dream+Theater/Images+and+Words"
                        .parse()
                        .unwrap(),
                    artist: ShortArtist {
                        name: "Dream Theater".into(),
                        mbid: "28503ab7-8bf2-4666-a7bd-2644bfc7cb1d".into(),
                        url: "http://www.last.fm/music/Dream+Theater".parse().unwrap()
                    },
                    images: vec![
                        i(ImageSize::Small, "..."),
                        i(ImageSize::Medium, "..."),
                        i(ImageSize::Large, "..."),
                    ]
                }]
            }
        );
    }

    #[test]
    fn user_get_top_artists() {
        let x: TopArtists = from_str(
            r#"<topartists user="RJ" type="overall">
  <artist rank="1">
    <name>Dream Theater</name>
    <playcount>1337</playcount>
    <mbid>28503ab7-8bf2-4666-a7bd-2644bfc7cb1d</mbid>
    <url>http://www.last.fm/music/Dream+Theater</url>
    <streamable>1</streamable>
    <image size="small">...</image>
    <image size="medium">...</image>
    <image size="large">...</image>
  </artist>
</topartists>"#,
        )
        .expect("can parse");
        assert_eq!(
            x,
            TopArtists {
                user: "RJ".into(),
                artists: vec![Artist {
                    rank: 1,
                    name: "Dream Theater".into(),
                    playcount: 1337,
                    mbid: "28503ab7-8bf2-4666-a7bd-2644bfc7cb1d".into(),
                    url: "http://www.last.fm/music/Dream+Theater".parse().unwrap(),
                    streamable: true,
                    images: vec![
                        i(ImageSize::Small, "..."),
                        i(ImageSize::Medium, "..."),
                        i(ImageSize::Large, "..."),
                    ]
                }]
            }
        )
    }

    #[test]
    fn display_period() {
        assert_eq!(Period::Overall.to_string(), "overall");
    }
}
