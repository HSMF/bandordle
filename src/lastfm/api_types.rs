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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename = "artist")]
pub struct Artist {
    name: String,
    mbid: String,
    url: Url,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageSize {
    Small,
    Medium,
    Large,
    Extralarge,
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
    pub artist: Artist,
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

pub type GetTopAlbumsResponse = LfmStatus<TopAlbums>;

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    use quick_xml::de::from_str;

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
        fn i(size: ImageSize, url: &str) -> Image {
            Image {
                size,
                url: url.into(),
            }
        }
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
                    artist: Artist {
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
}
