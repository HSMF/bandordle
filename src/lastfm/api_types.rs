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

    #[test]
    fn from_api() {
        let input = r##"<?xml version="1.0" encoding="UTF-8"?>
<lfm status="ok">
  <topalbums user="hydehsmf" page="1" perPage="50" totalPages="130" total="6477">
    <album rank="1">
      <name>Sundowning</name>
      <playcount>1462</playcount>
      <mbid>0c8ed9b3-0330-4139-b656-29b6bf746238</mbid>
      <url>https://www.last.fm/music/Sleep+Token/Sundowning</url>
      <artist>
        <name>Sleep Token</name>
        <mbid>775ad383-ac2d-479a-af9d-b561442cb749</mbid>
        <url>https://www.last.fm/music/Sleep+Token</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/e4dc7009e8281b2c178fb59c343051b2.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/e4dc7009e8281b2c178fb59c343051b2.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/e4dc7009e8281b2c178fb59c343051b2.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/e4dc7009e8281b2c178fb59c343051b2.jpg</image>
    </album>
    <album rank="2">
      <name>Take Me Back to Eden</name>
      <playcount>1414</playcount>
      <mbid>030c3a9c-abae-45a3-9063-0c78ed893a04</mbid>
      <url>https://www.last.fm/music/Sleep+Token/Take+Me+Back+to+Eden</url>
      <artist>
        <name>Sleep Token</name>
        <mbid>775ad383-ac2d-479a-af9d-b561442cb749</mbid>
        <url>https://www.last.fm/music/Sleep+Token</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/e7b531006b41eb9bb864362291962f39.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/e7b531006b41eb9bb864362291962f39.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/e7b531006b41eb9bb864362291962f39.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/e7b531006b41eb9bb864362291962f39.jpg</image>
    </album>
    <album rank="3">
      <name>This Place Will Become Your Tomb</name>
      <playcount>1198</playcount>
      <mbid>1793ac58-977e-4d6f-9fcc-fdcb046cc0a1</mbid>
      <url>https://www.last.fm/music/Sleep+Token/This+Place+Will+Become+Your+Tomb</url>
      <artist>
        <name>Sleep Token</name>
        <mbid>775ad383-ac2d-479a-af9d-b561442cb749</mbid>
        <url>https://www.last.fm/music/Sleep+Token</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/78786d2791a6e3d4fdc055f65e7f8455.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/78786d2791a6e3d4fdc055f65e7f8455.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/78786d2791a6e3d4fdc055f65e7f8455.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/78786d2791a6e3d4fdc055f65e7f8455.jpg</image>
    </album>
    <album rank="4">
      <name>Pitfalls</name>
      <playcount>742</playcount>
      <mbid>0a713dd9-7b15-4d17-a0fe-66b41948ff69</mbid>
      <url>https://www.last.fm/music/Leprous/Pitfalls</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/48b7bf032f0bae68e2d968f54ed72c04.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/48b7bf032f0bae68e2d968f54ed72c04.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/48b7bf032f0bae68e2d968f54ed72c04.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/48b7bf032f0bae68e2d968f54ed72c04.jpg</image>
    </album>
    <album rank="5">
      <name>Aphelion</name>
      <playcount>672</playcount>
      <mbid>184c734f-0640-4ebc-b2fa-84057c679ba9</mbid>
      <url>https://www.last.fm/music/Leprous/Aphelion</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/481b32d39f675139f9b45a13593f7567.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/481b32d39f675139f9b45a13593f7567.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/481b32d39f675139f9b45a13593f7567.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/481b32d39f675139f9b45a13593f7567.png</image>
    </album>
    <album rank="6">
      <name>Malina</name>
      <playcount>656</playcount>
      <mbid>22bc13c7-cf5e-4140-b550-74205d61b1af</mbid>
      <url>https://www.last.fm/music/Leprous/Malina</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/5c49e4e73884e291c3cf1bab329e163b.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/5c49e4e73884e291c3cf1bab329e163b.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/5c49e4e73884e291c3cf1bab329e163b.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/5c49e4e73884e291c3cf1bab329e163b.jpg</image>
    </album>
    <album rank="7">
      <name>Witness</name>
      <playcount>619</playcount>
      <mbid>1721c947-780b-41fd-a7e3-7bd6d73f84c4</mbid>
      <url>https://www.last.fm/music/VOLA/Witness</url>
      <artist>
        <name>VOLA</name>
        <mbid>fd529c0d-4a5c-479d-bbb8-601cefe2b38b</mbid>
        <url>https://www.last.fm/music/VOLA</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/24bc8816328f9948a4fcf3d5fcd3046b.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/24bc8816328f9948a4fcf3d5fcd3046b.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/24bc8816328f9948a4fcf3d5fcd3046b.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/24bc8816328f9948a4fcf3d5fcd3046b.png</image>
    </album>
    <album rank="8">
      <name>Even In Arcadia</name>
      <playcount>583</playcount>
      <mbid>5f3aff4e-e934-4cdc-b478-f74a8e246301</mbid>
      <url>https://www.last.fm/music/Sleep+Token/Even+In+Arcadia</url>
      <artist>
        <name>Sleep Token</name>
        <mbid>775ad383-ac2d-479a-af9d-b561442cb749</mbid>
        <url>https://www.last.fm/music/Sleep+Token</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/ca59461ca9b6b14cfc9c1183dbd82a8b.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/ca59461ca9b6b14cfc9c1183dbd82a8b.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/ca59461ca9b6b14cfc9c1183dbd82a8b.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/ca59461ca9b6b14cfc9c1183dbd82a8b.jpg</image>
    </album>
    <album rank="9">
      <name>The Congregation</name>
      <playcount>560</playcount>
      <mbid>1137487b-97f6-4aa7-a1ff-0287cb6e3624</mbid>
      <url>https://www.last.fm/music/Leprous/The+Congregation</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/3e4382394a13a0feb7266986224058bd.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/3e4382394a13a0feb7266986224058bd.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/3e4382394a13a0feb7266986224058bd.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/3e4382394a13a0feb7266986224058bd.jpg</image>
    </album>
    <album rank="10">
      <name>Eternal Blue</name>
      <playcount>542</playcount>
      <mbid>312248cc-12a5-46f4-8f9c-893df55deccf</mbid>
      <url>https://www.last.fm/music/Spiritbox/Eternal+Blue</url>
      <artist>
        <name>Spiritbox</name>
        <mbid>9c935736-7530-41e4-b776-1dbcf534c061</mbid>
        <url>https://www.last.fm/music/Spiritbox</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/a39e4dca10fbf0155a12c09724a7d20f.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/a39e4dca10fbf0155a12c09724a7d20f.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/a39e4dca10fbf0155a12c09724a7d20f.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/a39e4dca10fbf0155a12c09724a7d20f.jpg</image>
    </album>
    <album rank="11">
      <name>Melodies of Atonement</name>
      <playcount>521</playcount>
      <mbid>f6a1712d-e1fc-4fa4-95e5-472a231ac638</mbid>
      <url>https://www.last.fm/music/Leprous/Melodies+of+Atonement</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/b41e9251823f657d13c6e318c7e0b3da.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/b41e9251823f657d13c6e318c7e0b3da.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/b41e9251823f657d13c6e318c7e0b3da.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/b41e9251823f657d13c6e318c7e0b3da.png</image>
    </album>
    <album rank="12">
      <name>Applause Of A Distant Crowd</name>
      <playcount>521</playcount>
      <mbid>112854e7-4c83-4fa5-a078-edefd80bc31c</mbid>
      <url>https://www.last.fm/music/VOLA/Applause+Of+A+Distant+Crowd</url>
      <artist>
        <name>VOLA</name>
        <mbid>fd529c0d-4a5c-479d-bbb8-601cefe2b38b</mbid>
        <url>https://www.last.fm/music/VOLA</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/22d542a029a83834df41ae0e819da026.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/22d542a029a83834df41ae0e819da026.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/22d542a029a83834df41ae0e819da026.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/22d542a029a83834df41ae0e819da026.png</image>
    </album>
    <album rank="13">
      <name>Three Cheers for Sweet Revenge</name>
      <playcount>490</playcount>
      <mbid>0d7934da-7d3c-4cd6-9032-daf481026c45</mbid>
      <url>https://www.last.fm/music/My+Chemical+Romance/Three+Cheers+for+Sweet+Revenge</url>
      <artist>
        <name>My Chemical Romance</name>
        <mbid>c07f0676-9143-4217-8a9f-4c26bd636f13</mbid>
        <url>https://www.last.fm/music/My+Chemical+Romance</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/09cb27a9f908354fd210a07830951791.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/09cb27a9f908354fd210a07830951791.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/09cb27a9f908354fd210a07830951791.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/09cb27a9f908354fd210a07830951791.png</image>
    </album>
    <album rank="14">
      <name>Bilateral</name>
      <playcount>441</playcount>
      <mbid>44ac5d36-1497-407b-9bc0-3cbb0dad6e05</mbid>
      <url>https://www.last.fm/music/Leprous/Bilateral</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/cc7c082a48994f3e8297a457b0987ed6.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/cc7c082a48994f3e8297a457b0987ed6.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/cc7c082a48994f3e8297a457b0987ed6.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/cc7c082a48994f3e8297a457b0987ed6.png</image>
    </album>
    <album rank="15">
      <name>...and everything in between</name>
      <playcount>408</playcount>
      <mbid>4971125c-13a1-45ef-979d-c96b4dd3f8a1</mbid>
      <url>https://www.last.fm/music/Unprocessed/...and+everything+in+between</url>
      <artist>
        <name>Unprocessed</name>
        <mbid>ef3eb759-49b3-4f20-ac12-a92b87a3cb4c</mbid>
        <url>https://www.last.fm/music/Unprocessed</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/dbcc3452fd1a502b7fe69c71e83e891d.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/dbcc3452fd1a502b7fe69c71e83e891d.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/dbcc3452fd1a502b7fe69c71e83e891d.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/dbcc3452fd1a502b7fe69c71e83e891d.jpg</image>
    </album>
    <album rank="16">
      <name>THE DEATH OF PEACE OF MIND</name>
      <playcount>371</playcount>
      <mbid>dd144a37-f8e3-4e14-aeda-83fbaba90f17</mbid>
      <url>https://www.last.fm/music/Bad+Omens/THE+DEATH+OF+PEACE+OF+MIND</url>
      <artist>
        <name>Bad Omens</name>
        <mbid>eecada09-acfc-472d-ae55-e9e5a43f12d8</mbid>
        <url>https://www.last.fm/music/Bad+Omens</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/31247a4412911ac3e8502777d456a5c7.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/31247a4412911ac3e8502777d456a5c7.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/31247a4412911ac3e8502777d456a5c7.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/31247a4412911ac3e8502777d456a5c7.jpg</image>
    </album>
    <album rank="17">
      <name>Pain Remains</name>
      <playcount>358</playcount>
      <mbid></mbid>
      <url>https://www.last.fm/music/Lorna+Shore/Pain+Remains</url>
      <artist>
        <name>Lorna Shore</name>
        <mbid>e86fc1f5-d632-44b2-8aea-38f83aadffe8</mbid>
        <url>https://www.last.fm/music/Lorna+Shore</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/b7c188120ce3e77e138ffd8bdae33d55.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/b7c188120ce3e77e138ffd8bdae33d55.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/b7c188120ce3e77e138ffd8bdae33d55.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/b7c188120ce3e77e138ffd8bdae33d55.jpg</image>
    </album>
    <album rank="18">
      <name>Friend Of A Phantom</name>
      <playcount>347</playcount>
      <mbid>7a35076c-9062-41f9-9651-141ad53e92a5</mbid>
      <url>https://www.last.fm/music/VOLA/Friend+Of+A+Phantom</url>
      <artist>
        <name>VOLA</name>
        <mbid>fd529c0d-4a5c-479d-bbb8-601cefe2b38b</mbid>
        <url>https://www.last.fm/music/VOLA</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/727eca7c069184001bcee782a780c54e.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/727eca7c069184001bcee782a780c54e.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/727eca7c069184001bcee782a780c54e.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/727eca7c069184001bcee782a780c54e.png</image>
    </album>
    <album rank="19">
      <name>Coal</name>
      <playcount>336</playcount>
      <mbid>19a658e8-812a-48bc-8677-eec8263a4968</mbid>
      <url>https://www.last.fm/music/Leprous/Coal</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/c8345296e6484c76b15a55df964ed50a.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/c8345296e6484c76b15a55df964ed50a.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/c8345296e6484c76b15a55df964ed50a.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/c8345296e6484c76b15a55df964ed50a.png</image>
    </album>
    <album rank="20">
      <name>The Poetic Edda</name>
      <playcount>324</playcount>
      <mbid></mbid>
      <url>https://www.last.fm/music/Synestia/The+Poetic+Edda</url>
      <artist>
        <name>Synestia</name>
        <mbid></mbid>
        <url>https://www.last.fm/music/Synestia</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/37d8b98d763068da5b31c8b4bf39776a.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/37d8b98d763068da5b31c8b4bf39776a.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/37d8b98d763068da5b31c8b4bf39776a.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/37d8b98d763068da5b31c8b4bf39776a.jpg</image>
    </album>
    <album rank="21">
      <name>Corporation</name>
      <playcount>309</playcount>
      <mbid>c5bfafc9-9eb1-4719-98e3-2302eb602e96</mbid>
      <url>https://www.last.fm/music/Aviana/Corporation</url>
      <artist>
        <name>Aviana</name>
        <mbid>2bbc7a00-ead2-42cb-85ae-1d666893e7ab</mbid>
        <url>https://www.last.fm/music/Aviana</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/ab0ae8d7bd3e77cf9a599840d2df12b8.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/ab0ae8d7bd3e77cf9a599840d2df12b8.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/ab0ae8d7bd3e77cf9a599840d2df12b8.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/ab0ae8d7bd3e77cf9a599840d2df12b8.jpg</image>
    </album>
    <album rank="22">
      <name>Two</name>
      <playcount>300</playcount>
      <mbid>90698f24-dde9-497b-bc40-514887c4d98a</mbid>
      <url>https://www.last.fm/music/Sleep+Token/Two</url>
      <artist>
        <name>Sleep Token</name>
        <mbid>775ad383-ac2d-479a-af9d-b561442cb749</mbid>
        <url>https://www.last.fm/music/Sleep+Token</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/613cadb2ca6509670927586b007a6b2f.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/613cadb2ca6509670927586b007a6b2f.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/613cadb2ca6509670927586b007a6b2f.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/613cadb2ca6509670927586b007a6b2f.jpg</image>
    </album>
    <album rank="23">
      <name>Union</name>
      <playcount>289</playcount>
      <mbid>2660059c-bbff-4400-a365-90d32717034b</mbid>
      <url>https://www.last.fm/music/Ihlo/Union</url>
      <artist>
        <name>Ihlo</name>
        <mbid>4ab40b83-0b41-46af-b330-7162df3512ea</mbid>
        <url>https://www.last.fm/music/Ihlo</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/de6c29379ddbf220d77fdd1e8b76919d.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/de6c29379ddbf220d77fdd1e8b76919d.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/de6c29379ddbf220d77fdd1e8b76919d.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/de6c29379ddbf220d77fdd1e8b76919d.jpg</image>
    </album>
    <album rank="24">
      <name>Tall Poppy Syndrome</name>
      <playcount>289</playcount>
      <mbid>7f51ee89-3604-4e7a-84bd-4c14ff5f9e32</mbid>
      <url>https://www.last.fm/music/Leprous/Tall+Poppy+Syndrome</url>
      <artist>
        <name>Leprous</name>
        <mbid>a2e55cf5-ca3a-4c26-ba62-fc4a4f2bc603</mbid>
        <url>https://www.last.fm/music/Leprous</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/7726498700ae4e81b8102d36e9898acd.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/7726498700ae4e81b8102d36e9898acd.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/7726498700ae4e81b8102d36e9898acd.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/7726498700ae4e81b8102d36e9898acd.png</image>
    </album>
    <album rank="25">
      <name>Ascension</name>
      <playcount>287</playcount>
      <mbid>82bff9da-5c5d-446b-8709-5eaadb44ffa5</mbid>
      <url>https://www.last.fm/music/Mirar/Ascension</url>
      <artist>
        <name>Mirar</name>
        <mbid>1c79c8c5-a91c-4450-8c43-22a3834798e4</mbid>
        <url>https://www.last.fm/music/Mirar</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/8f146027bf887912916f7499fc76ddaa.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/8f146027bf887912916f7499fc76ddaa.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/8f146027bf887912916f7499fc76ddaa.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/8f146027bf887912916f7499fc76ddaa.jpg</image>
    </album>
    <album rank="26">
      <name>CURSED</name>
      <playcount>285</playcount>
      <mbid>2723e949-76f5-4bf8-bcdf-9ff12cd1d4d2</mbid>
      <url>https://www.last.fm/music/PALEFACE+SWISS/CURSED</url>
      <artist>
        <name>PALEFACE SWISS</name>
        <mbid>6be0e54b-d3ff-4c17-b7e6-f82d0f624a47</mbid>
        <url>https://www.last.fm/music/PALEFACE+SWISS</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/36ec36ea0e541f2857b2ab8f9b05e4a8.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/36ec36ea0e541f2857b2ab8f9b05e4a8.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/36ec36ea0e541f2857b2ab8f9b05e4a8.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/36ec36ea0e541f2857b2ab8f9b05e4a8.jpg</image>
    </album>
    <album rank="27">
      <name>Zeal &amp; Ardor</name>
      <playcount>284</playcount>
      <mbid>d308adb3-9c27-49d7-af52-6471639712cb</mbid>
      <url>https://www.last.fm/music/Zeal+&amp;+Ardor/Zeal+&amp;+Ardor</url>
      <artist>
        <name>Zeal &amp; Ardor</name>
        <mbid>5e90193b-5183-406b-9f3a-b66bb66daccb</mbid>
        <url>https://www.last.fm/music/Zeal+&amp;+Ardor</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/811bb109876a51a9afb66c0ab4c22c90.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/811bb109876a51a9afb66c0ab4c22c90.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/811bb109876a51a9afb66c0ab4c22c90.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/811bb109876a51a9afb66c0ab4c22c90.png</image>
    </album>
    <album rank="28">
      <name>The Way It Ends</name>
      <playcount>279</playcount>
      <mbid>8baa85d8-c203-4797-948b-ee28efef34f9</mbid>
      <url>https://www.last.fm/music/Currents/The+Way+It+Ends</url>
      <artist>
        <name>Currents</name>
        <mbid>b7e442b4-30cd-4bd2-9238-032e74f7807d</mbid>
        <url>https://www.last.fm/music/Currents</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/cd076fd5dc644643ea989f3179bcb1d8.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/cd076fd5dc644643ea989f3179bcb1d8.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/cd076fd5dc644643ea989f3179bcb1d8.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/cd076fd5dc644643ea989f3179bcb1d8.jpg</image>
    </album>
    <album rank="29">
      <name>Inmazes</name>
      <playcount>273</playcount>
      <mbid>10a42241-f2ec-4d36-8486-5713b0cdddd2</mbid>
      <url>https://www.last.fm/music/VOLA/Inmazes</url>
      <artist>
        <name>VOLA</name>
        <mbid>fd529c0d-4a5c-479d-bbb8-601cefe2b38b</mbid>
        <url>https://www.last.fm/music/VOLA</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/e6548147aa3341b9c1ad892a5e085e29.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/e6548147aa3341b9c1ad892a5e085e29.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/e6548147aa3341b9c1ad892a5e085e29.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/e6548147aa3341b9c1ad892a5e085e29.jpg</image>
    </album>
    <album rank="30">
      <name>Phanerozoic II: Mesozoic | Cenozoic</name>
      <playcount>267</playcount>
      <mbid>61ee9ed4-8cf0-4635-b400-2613af345536</mbid>
      <url>https://www.last.fm/music/The+Ocean/Phanerozoic+II:+Mesozoic+%7C+Cenozoic</url>
      <artist>
        <name>The Ocean</name>
        <mbid>8d0acf0e-c099-49ac-b4b3-d57ca9eb2561</mbid>
        <url>https://www.last.fm/music/The+Ocean</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/08e489c18d36b6df3000b54768a89e7c.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/08e489c18d36b6df3000b54768a89e7c.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/08e489c18d36b6df3000b54768a89e7c.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/08e489c18d36b6df3000b54768a89e7c.png</image>
    </album>
    <album rank="31">
      <name>People Watching</name>
      <playcount>264</playcount>
      <mbid>c2e0f98f-700a-4b03-80e7-f87210e59a84</mbid>
      <url>https://www.last.fm/music/156%2FSilence/People+Watching</url>
      <artist>
        <name>156/Silence</name>
        <mbid>c4a3dc14-e745-4ca8-af27-e89ff19232d3</mbid>
        <url>https://www.last.fm/music/156%2FSilence</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/d3dd39683134ba883222ef8875f61ed4.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/d3dd39683134ba883222ef8875f61ed4.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/d3dd39683134ba883222ef8875f61ed4.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/d3dd39683134ba883222ef8875f61ed4.jpg</image>
    </album>
    <album rank="32">
      <name>Remember That You Will Die</name>
      <playcount>262</playcount>
      <mbid>34bcb239-ae92-409a-82a0-2aa74a591ec5</mbid>
      <url>https://www.last.fm/music/Polyphia/Remember+That+You+Will+Die</url>
      <artist>
        <name>Polyphia</name>
        <mbid>344bfb00-27f0-4ff2-b96b-048ed1c6a968</mbid>
        <url>https://www.last.fm/music/Polyphia</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/b7562e45ab388882d6dec1767e69769d.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/b7562e45ab388882d6dec1767e69769d.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/b7562e45ab388882d6dec1767e69769d.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/b7562e45ab388882d6dec1767e69769d.jpg</image>
    </album>
    <album rank="33">
      <name>In Contact</name>
      <playcount>249</playcount>
      <mbid>41a0b0da-9312-4f5b-bba7-33c19f54b833</mbid>
      <url>https://www.last.fm/music/Caligula%27s+Horse/In+Contact</url>
      <artist>
        <name>Caligula&apos;s Horse</name>
        <mbid>3ab39ae8-599e-4d70-8902-894dcfcf4092</mbid>
        <url>https://www.last.fm/music/Caligula%27s+Horse</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/610ca54a55e1979d4e1264314b899aa3.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/610ca54a55e1979d4e1264314b899aa3.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/610ca54a55e1979d4e1264314b899aa3.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/610ca54a55e1979d4e1264314b899aa3.jpg</image>
    </album>
    <album rank="34">
      <name>The Last Spell (Original Game Soundtrack)</name>
      <playcount>248</playcount>
      <mbid>8eeccaee-95b6-4311-9727-bb37f1d599ce</mbid>
      <url>https://www.last.fm/music/Remi+Gallego/The+Last+Spell+(Original+Game+Soundtrack)</url>
      <artist>
        <name>Remi Gallego</name>
        <mbid></mbid>
        <url>https://www.last.fm/music/Remi+Gallego</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/2d09e6712b7a20872777dee39978acbd.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/2d09e6712b7a20872777dee39978acbd.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/2d09e6712b7a20872777dee39978acbd.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/2d09e6712b7a20872777dee39978acbd.jpg</image>
    </album>
    <album rank="35">
      <name>Mare</name>
      <playcount>245</playcount>
      <mbid>327e21fa-dc5f-40aa-9269-d3eea0dfc999</mbid>
      <url>https://www.last.fm/music/Mirar/Mare</url>
      <artist>
        <name>Mirar</name>
        <mbid>1c79c8c5-a91c-4450-8c43-22a3834798e4</mbid>
        <url>https://www.last.fm/music/Mirar</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/3d7e76dc9dd74fe47f081c812fba8848.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/3d7e76dc9dd74fe47f081c812fba8848.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/3d7e76dc9dd74fe47f081c812fba8848.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/3d7e76dc9dd74fe47f081c812fba8848.jpg</image>
    </album>
    <album rank="36">
      <name>The Black Parade</name>
      <playcount>243</playcount>
      <mbid>16eb8908-7d05-48b9-af7f-cb017302482a</mbid>
      <url>https://www.last.fm/music/My+Chemical+Romance/The+Black+Parade</url>
      <artist>
        <name>My Chemical Romance</name>
        <mbid>c07f0676-9143-4217-8a9f-4c26bd636f13</mbid>
        <url>https://www.last.fm/music/My+Chemical+Romance</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/7675defb2787ce67cd030081eb8ff77c.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/7675defb2787ce67cd030081eb8ff77c.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/7675defb2787ce67cd030081eb8ff77c.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/7675defb2787ce67cd030081eb8ff77c.png</image>
    </album>
    <album rank="37">
      <name>One</name>
      <playcount>242</playcount>
      <mbid>6d931693-9bd8-4a1b-bf5a-15d4bd2e1949</mbid>
      <url>https://www.last.fm/music/Sleep+Token/One</url>
      <artist>
        <name>Sleep Token</name>
        <mbid>775ad383-ac2d-479a-af9d-b561442cb749</mbid>
        <url>https://www.last.fm/music/Sleep+Token</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/f8d9d43fb607d50e748bf48f27ce8788.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/f8d9d43fb607d50e748bf48f27ce8788.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/f8d9d43fb607d50e748bf48f27ce8788.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/f8d9d43fb607d50e748bf48f27ce8788.jpg</image>
    </album>
    <album rank="38">
      <name>A Way Out</name>
      <playcount>238</playcount>
      <mbid>21878829-7f20-4402-893e-27c35e18d1d1</mbid>
      <url>https://www.last.fm/music/Distorted+Harmony/A+Way+Out</url>
      <artist>
        <name>Distorted Harmony</name>
        <mbid>ff230c4c-9557-4576-b652-2263c51a48b1</mbid>
        <url>https://www.last.fm/music/Distorted+Harmony</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/d2607c5f1def067c04412064246b5cad.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/d2607c5f1def067c04412064246b5cad.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/d2607c5f1def067c04412064246b5cad.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/d2607c5f1def067c04412064246b5cad.jpg</image>
    </album>
    <album rank="39">
      <name>Handmade Cities</name>
      <playcount>235</playcount>
      <mbid>7dc22510-b792-4d94-8b38-3b49c6500659</mbid>
      <url>https://www.last.fm/music/Plini/Handmade+Cities</url>
      <artist>
        <name>Plini</name>
        <mbid>3f6c0aa1-a7a9-4ff2-9c50-e84d4b0de178</mbid>
        <url>https://www.last.fm/music/Plini</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/9c099708529ef0d31f0a09c1f3c1f348.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/9c099708529ef0d31f0a09c1f3c1f348.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/9c099708529ef0d31f0a09c1f3c1f348.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/9c099708529ef0d31f0a09c1f3c1f348.jpg</image>
    </album>
    <album rank="40">
      <name>Charcoal Grace</name>
      <playcount>233</playcount>
      <mbid>2ce11dec-5e29-49f2-8b68-db6221439400</mbid>
      <url>https://www.last.fm/music/Caligula%27s+Horse/Charcoal+Grace</url>
      <artist>
        <name>Caligula&apos;s Horse</name>
        <mbid>3ab39ae8-599e-4d70-8902-894dcfcf4092</mbid>
        <url>https://www.last.fm/music/Caligula%27s+Horse</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/78e683d95ce939b5bc06c54e3702bb8c.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/78e683d95ce939b5bc06c54e3702bb8c.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/78e683d95ce939b5bc06c54e3702bb8c.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/78e683d95ce939b5bc06c54e3702bb8c.png</image>
    </album>
    <album rank="41">
      <name>Fear &amp; Dagger</name>
      <playcount>221</playcount>
      <mbid></mbid>
      <url>https://www.last.fm/music/PALEFACE+SWISS/Fear+&amp;+Dagger</url>
      <artist>
        <name>PALEFACE SWISS</name>
        <mbid>6be0e54b-d3ff-4c17-b7e6-f82d0f624a47</mbid>
        <url>https://www.last.fm/music/PALEFACE+SWISS</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/1fab93c6f2bb22c0d37d1fbd38c192ef.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/1fab93c6f2bb22c0d37d1fbd38c192ef.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/1fab93c6f2bb22c0d37d1fbd38c192ef.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/1fab93c6f2bb22c0d37d1fbd38c192ef.jpg</image>
    </album>
    <album rank="42">
      <name>Impulse Voices</name>
      <playcount>214</playcount>
      <mbid>4bf64586-26c9-4a61-a8f4-ee808fb23b0f</mbid>
      <url>https://www.last.fm/music/Plini/Impulse+Voices</url>
      <artist>
        <name>Plini</name>
        <mbid>3f6c0aa1-a7a9-4ff2-9c50-e84d4b0de178</mbid>
        <url>https://www.last.fm/music/Plini</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/9c236e47038f785d01b0d9da2eeec638.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/9c236e47038f785d01b0d9da2eeec638.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/9c236e47038f785d01b0d9da2eeec638.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/9c236e47038f785d01b0d9da2eeec638.png</image>
    </album>
    <album rank="43">
      <name>Rise Radiant (Bonus Tracks Version)</name>
      <playcount>206</playcount>
      <mbid></mbid>
      <url>https://www.last.fm/music/Caligula%27s+Horse/Rise+Radiant+(Bonus+Tracks+Version)</url>
      <artist>
        <name>Caligula&apos;s Horse</name>
        <mbid>3ab39ae8-599e-4d70-8902-894dcfcf4092</mbid>
        <url>https://www.last.fm/music/Caligula%27s+Horse</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/b4a271c524ecd815c5ab0f464d42ca3b.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/b4a271c524ecd815c5ab0f464d42ca3b.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/b4a271c524ecd815c5ab0f464d42ca3b.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/b4a271c524ecd815c5ab0f464d42ca3b.png</image>
    </album>
    <album rank="44">
      <name>Holocene</name>
      <playcount>206</playcount>
      <mbid>0a1346f1-bd82-4a66-b6d1-c01954567641</mbid>
      <url>https://www.last.fm/music/The+Ocean/Holocene</url>
      <artist>
        <name>The Ocean</name>
        <mbid>8d0acf0e-c099-49ac-b4b3-d57ca9eb2561</mbid>
        <url>https://www.last.fm/music/The+Ocean</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/b794b164e830e935d124823186327c0f.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/b794b164e830e935d124823186327c0f.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/b794b164e830e935d124823186327c0f.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/b794b164e830e935d124823186327c0f.jpg</image>
    </album>
    <album rank="45">
      <name>Bloom</name>
      <playcount>204</playcount>
      <mbid>43fa4fde-ccec-4fc1-81dc-5c651671353a</mbid>
      <url>https://www.last.fm/music/Caligula%27s+Horse/Bloom</url>
      <artist>
        <name>Caligula&apos;s Horse</name>
        <mbid>3ab39ae8-599e-4d70-8902-894dcfcf4092</mbid>
        <url>https://www.last.fm/music/Caligula%27s+Horse</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/98be46f47394f12bd9f5a5f9f9089908.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/98be46f47394f12bd9f5a5f9f9089908.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/98be46f47394f12bd9f5a5f9f9089908.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/98be46f47394f12bd9f5a5f9f9089908.png</image>
    </album>
    <album rank="46">
      <name>GREIF</name>
      <playcount>198</playcount>
      <mbid></mbid>
      <url>https://www.last.fm/music/Zeal+&amp;+Ardor/GREIF</url>
      <artist>
        <name>Zeal &amp; Ardor</name>
        <mbid>5e90193b-5183-406b-9f3a-b66bb66daccb</mbid>
        <url>https://www.last.fm/music/Zeal+&amp;+Ardor</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/494a550cde36cac86d0d636e62442486.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/494a550cde36cac86d0d636e62442486.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/494a550cde36cac86d0d636e62442486.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/494a550cde36cac86d0d636e62442486.png</image>
    </album>
    <album rank="47">
      <name>Technatura</name>
      <playcount>195</playcount>
      <mbid>2c50957d-d7f8-4b77-a4e8-716f7ca87d4a</mbid>
      <url>https://www.last.fm/music/Vulkan/Technatura</url>
      <artist>
        <name>Vulkan</name>
        <mbid>58c28ddb-c475-4939-9e95-e97a17f62aac</mbid>
        <url>https://www.last.fm/music/Vulkan</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/b04dd09e7b959ca42a827fc25fc165e8.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/b04dd09e7b959ca42a827fc25fc165e8.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/b04dd09e7b959ca42a827fc25fc165e8.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/b04dd09e7b959ca42a827fc25fc165e8.jpg</image>
    </album>
    <album rank="48">
      <name>Manic</name>
      <playcount>192</playcount>
      <mbid>067cb4ff-70d2-4bb6-8501-0fed6e26fff3</mbid>
      <url>https://www.last.fm/music/Wage+War/Manic</url>
      <artist>
        <name>Wage War</name>
        <mbid>653f3412-63c8-4bd8-bce1-a33aaf93a837</mbid>
        <url>https://www.last.fm/music/Wage+War</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/7ef8d38b2e1c4fab7f82c4c56aa1c876.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/7ef8d38b2e1c4fab7f82c4c56aa1c876.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/7ef8d38b2e1c4fab7f82c4c56aa1c876.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/7ef8d38b2e1c4fab7f82c4c56aa1c876.jpg</image>
    </album>
    <album rank="49">
      <name>Hybrid Theory (Bonus Edition)</name>
      <playcount>190</playcount>
      <mbid></mbid>
      <url>https://www.last.fm/music/Linkin+Park/Hybrid+Theory+(Bonus+Edition)</url>
      <artist>
        <name>Linkin Park</name>
        <mbid>f59c5520-5f46-4d2c-b2c4-822eabf53419</mbid>
        <url>https://www.last.fm/music/Linkin+Park</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/56989cdb558cb4f6609eb906029399d8.jpg</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/56989cdb558cb4f6609eb906029399d8.jpg</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/56989cdb558cb4f6609eb906029399d8.jpg</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/56989cdb558cb4f6609eb906029399d8.jpg</image>
    </album>
    <album rank="50">
      <name>The Joy of Motion</name>
      <playcount>187</playcount>
      <mbid>c3549f65-4cab-4381-a382-61e1d033dd2c</mbid>
      <url>https://www.last.fm/music/Animals+as+Leaders/The+Joy+of+Motion</url>
      <artist>
        <name>Animals as Leaders</name>
        <mbid>5c2d2520-950b-4c78-84fc-78a9328172a3</mbid>
        <url>https://www.last.fm/music/Animals+as+Leaders</url>
      </artist>
      <image size="small">https://lastfm.freetls.fastly.net/i/u/34s/419579a7f5314d95cd864b946bb31772.png</image>
      <image size="medium">https://lastfm.freetls.fastly.net/i/u/64s/419579a7f5314d95cd864b946bb31772.png</image>
      <image size="large">https://lastfm.freetls.fastly.net/i/u/174s/419579a7f5314d95cd864b946bb31772.png</image>
      <image size="extralarge">https://lastfm.freetls.fastly.net/i/u/300x300/419579a7f5314d95cd864b946bb31772.png</image>
    </album>
  </topalbums>
</lfm>"##;

        quick_xml::de::from_str::<GetTopAlbumsResponse>(input).expect("can parse");
    }
}
