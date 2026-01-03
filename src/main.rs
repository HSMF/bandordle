use std::{
    collections::{HashMap, HashSet},
    env,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{
        HeaderValue, StatusCode,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey as _, VerifyWithKey as _};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use sha2::Sha512;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use ts_rs::TS;
use uuid::Uuid;

const MAX_GUESSES: usize = 6;

struct Config {
    lastfm_apikey: String,
    auth_callback_url: String,
    jwt_key: Hmac<Sha512>,
}

#[derive(Clone)]
struct WordList(&'static [HashSet<&'static str>]);

#[derive(Clone)]
struct SharedState {
    mutable: Arc<RwLock<AppState>>,
    config: Arc<Config>,
    pool: SqlitePool,
    lastfm: Arc<lastfm::Client>,

    word_list: WordList,
}

#[derive(Default)]
struct AppState {
    db: HashMap<Uuid, Mutex<SessionState>>,
}

#[derive(Clone)]
struct SessionState {
    words: Vec<String>,
    num_guesses: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("no such session")]
    NoSession,
    #[error("I don't know the word {0}")]
    UnknownWord(String),
    #[error("{0}")]
    GradingError(GradingError),
    #[error("something went wrong while contacting LastFM: {0}")]
    LastFm(lastfm::Error),
    #[error("missing parameter {0}")]
    MissingParam(&'static str),
    #[error("user has no albums")]
    NoAlbums,
    #[error("too many guesses")]
    TooManyGuesses,
    #[error("internal server error")]
    Internal(Box<dyn std::error::Error>),
    #[error("no user to fetch data for")]
    NoUser,
}

impl AppError {
    fn internal<E>(e: E) -> AppError
    where
        E: std::error::Error + 'static,
    {
        AppError::Internal(Box::new(e))
    }
}

impl From<lastfm::Error> for AppError {
    fn from(value: lastfm::Error) -> Self {
        AppError::LastFm(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let status = match &self {
            AppError::NoSession => StatusCode::NOT_FOUND,
            AppError::UnknownWord(..)
            | AppError::NoAlbums
            | AppError::NoUser
            | AppError::MissingParam(..)
            | AppError::GradingError(..) => StatusCode::BAD_REQUEST,
            AppError::TooManyGuesses => StatusCode::FORBIDDEN,
            AppError::LastFm(_) | AppError::Internal(..) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if let AppError::Internal(ref e) = self {
            log::error!("interal server error {e}");
        }

        (
            status,
            Json(ErrorResponse {
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(
    Debug, PartialEq, Eq, Default, Clone, Copy, Hash, PartialOrd, Ord, Serialize, Deserialize, TS,
)]
pub enum Grade {
    #[default]
    Incorrect,
    WrongPlace,
    Correct,
}

#[derive(thiserror::Error, Debug, Serialize, TS)]
pub enum GradingError {
    #[error("Wrong length (expected {0}, have {1})")]
    WrongLength(usize, usize),
    #[error("Wrong number of words (expected {0}, have {1})")]
    WrongNumberOfWords(usize, usize),
}

fn grade(expected: &str, guess: &str) -> Result<Vec<Grade>, GradingError> {
    if expected.len() != guess.len() {
        return Err(GradingError::WrongLength(expected.len(), guess.len()));
    }

    let mut word: Vec<_> = guess.chars().map(Some).collect();
    let mut expected: Vec<_> = expected.chars().map(Some).collect();

    let mut ret = vec![Grade::Incorrect; expected.len()];

    for (i, (w, e)) in word.iter_mut().zip(expected.iter_mut()).enumerate() {
        if w == e {
            ret[i] = Grade::Correct;
            *w = None;
            *e = None;
        }
    }

    for (i, w) in word.iter().enumerate() {
        if w.is_none() {
            continue;
        }
        for e in expected.iter_mut() {
            if w == e {
                ret[i] = Grade::WrongPlace;
                *e = None;
                break;
            }
        }
    }

    Ok(ret)
}

impl SessionState {
    fn new(words: Vec<String>) -> Self {
        Self {
            words,
            num_guesses: 0,
        }
    }
}

impl WordList {
    fn new(path: impl AsRef<Path>) -> Self {
        let s = std::fs::read_to_string(path).expect("cannot read wordlist");

        let max_len = s.lines().map(|x| x.len()).max().expect("wordlist is empty");

        let s = s.to_lowercase().leak();

        let mut v = vec![HashSet::new(); max_len + 1];

        for line in s.lines() {
            v[line.len()].insert(line);
        }

        Self(Box::leak(v.into_boxed_slice()))
    }

    fn contains(&self, w: &str) -> bool {
        self.0.get(w.len()).is_some_and(|list| list.contains(w))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().expect("have dotenv");
    fn var(name: &str) -> String {
        env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
    }

    let config = Arc::new(Config {
        lastfm_apikey: var("LASTFM_APIKEY"),
        auth_callback_url: var("AUTH_CALLBACK_URL"),
        jwt_key: Hmac::new_from_slice(var("JWT_KEY").as_bytes()).expect("create new key"),
    });
    let mutable = Default::default();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is set");
    let pool = SqlitePoolOptions::new()
        .connect(&database_url)
        .await
        .expect("can connect to db");
    let lastfm = Arc::new(lastfm::Client::new(
        var("LASTFM_SHARED_SECRET"),
        var("LASTFM_APIKEY"),
    ));
    let state = SharedState {
        mutable: Arc::clone(&mutable),
        config: Arc::clone(&config),
        pool,
        lastfm,
        word_list: WordList::new("./wordlist.txt"),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/newgame", post(newgame))
        .route("/api/v1/newgame-album", post(newgame_album))
        .route("/api/v1/guess", post(guess))
        .route("/api/v1/top-albums", get(get_top_albums))
        .route("/api/v1/signin", get(signin))
        .route("/api/v1/auth-url", get(get_auth_url))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .allow_credentials(true)
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_auth_url(State(state): State<SharedState>) -> String {
    let api_key = &state.config.lastfm_apikey;
    let cb = &state.config.auth_callback_url;
    format!("http://www.last.fm/api/auth/?api_key={api_key}&cb={cb}")
}

#[derive(Serialize, Deserialize)]
struct JwtClaims {
    fmname: String,
    exp: i64,
}

fn sign_cookie(
    key: &Hmac<Sha512>,
    username: String,
    days: i64,
    jar: CookieJar,
) -> Result<CookieJar, AppError> {
    let mut t = time::OffsetDateTime::now_utc();
    t += time::Duration::days(days);
    let claim = JwtClaims {
        fmname: username,
        exp: t.unix_timestamp(),
    };
    let session = claim.sign_with_key(key).map_err(AppError::internal)?;

    let cookie = Cookie::build(("session", session)).path("/").expires(t);

    let jar = jar.add(cookie);

    Ok(jar)
}

#[derive(Deserialize)]
struct SigninQuery {
    token: String,
}
async fn signin(
    State(state): State<SharedState>,
    Query(query): Query<SigninQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let token = query.token;

    let session = state
        .lastfm
        .authenticate(&token)
        .await
        .map_err(AppError::LastFm)?;

    sqlx::query!(
        "INSERT INTO user
            (lastfm_name, lastfm_key, auth_at, lastfm_subscriber)
        VALUES (
            ?, ?, unixepoch(), ?
        );
        ",
        session.name,
        session.key,
        session.subscriber
    )
    .execute(&state.pool)
    .await
    .map_err(|x| AppError::Internal(Box::new(x)))?;

    let jar = sign_cookie(&state.config.jwt_key, session.name, 4 * 7, jar)?;

    Ok((jar, Redirect::to("/")))
}

#[axum::debug_handler]
async fn get_top_albums(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let x = state
        .lastfm
        .top_albums(query.get("user").ok_or(AppError::MissingParam("user"))?)
        .send();
    x.await.map_err(AppError::LastFm).map(Json)
}

#[derive(Serialize, TS)]
#[ts(export)]
struct NewGameResult {
    id: Uuid,
    len: Vec<usize>,
}

fn pick_word(it: impl IntoIterator<Item = String>) -> Result<(Vec<String>, Vec<usize>), AppError> {
    let mut rng = rand::rng();
    let word = it
        .into_iter()
        .filter(|word| {
            word.chars()
                .any(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9'))
        })
        .choose(&mut rng)
        .ok_or(AppError::NoAlbums)?;
    let word: String = word
        .chars()
        .filter_map(|ch| match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            'A'..='Z' => Some(ch.to_ascii_lowercase()),
            ch if ch.is_whitespace() => Some(ch),
            _ => None,
        })
        .collect();
    let words: Vec<_> = word.split_whitespace().map(ToOwned::to_owned).collect();
    let len = words.iter().map(|x| x.len()).collect();
    Ok((words, len))
}

// TODO: temporary until we have users
#[derive(Serialize, Deserialize)]
struct NewGameQuery {
    user: Option<String>,
}

async fn newgame(
    jar: CookieJar,
    Query(query): Query<NewGameQuery>,
    State(state): State<SharedState>,
) -> Result<Json<NewGameResult>, AppError> {
    // TODO: user middleware
    let user = query.user.or(jar.get("session").and_then(|session| {
        session
            .value()
            .verify_with_key(&state.config.jwt_key)
            .ok()
            .map(|claims: JwtClaims| claims.fmname)
    }));
    let user = user.ok_or(AppError::NoUser)?;
    log::info!("creating new game (artist) for {user}");
    let resp = state
        .lastfm
        .top_artists(&user)
        .send()
        .await
        .map_err(AppError::LastFm)?;
    let (words, len) = pick_word(resp.artists.into_iter().map(|x| x.name))?;

    let id = Uuid::new_v4();
    let state = &mut state.mutable.write().unwrap();

    state.db.insert(id, Mutex::new(SessionState::new(words)));
    Ok(Json(NewGameResult { id, len }))
}

async fn newgame_album(State(state): State<SharedState>) -> Result<Json<NewGameResult>, AppError> {
    log::info!("creating new game (album)");
    let resp = state
        .lastfm
        .top_albums("hydehsmf")
        .send()
        .await
        .map_err(AppError::LastFm)?;
    let (words, len) = pick_word(resp.albums.into_iter().map(|x| x.name))?;

    let id = Uuid::new_v4();
    let state = &mut state.mutable.write().unwrap();

    state.db.insert(id, Mutex::new(SessionState::new(words)));
    Ok(Json(NewGameResult { id, len }))
}

#[derive(Deserialize, TS)]
#[ts(export)]
struct GuessArgs {
    id: Uuid,
    guess: String,
}
#[derive(Serialize, TS)]
#[ts(export)]
struct GuessResult {
    grade: Vec<Vec<Grade>>,
}

async fn guess(
    State(full_state): State<SharedState>,
    Json(guess): Json<GuessArgs>,
) -> Result<Json<GuessResult>, AppError> {
    fn inner(
        full_state: &SharedState,
        guess: GuessArgs,
        should_delete: &mut bool,
    ) -> Result<GuessResult, AppError> {
        let words: Vec<_> = guess.guess.split_whitespace().collect();
        let st = full_state.mutable.read().unwrap();
        let state = st.db.get(&guess.id).ok_or(AppError::NoSession)?;
        let mut state = state.lock().unwrap();

        if state.words.len() != words.len() {
            return Err(AppError::GradingError(GradingError::WrongNumberOfWords(
                state.words.len(),
                words.len(),
            )));
        }

        let grade = state
            .words
            .iter()
            .zip(words)
            .map(|(expected, word)| {
                if expected != word && !full_state.word_list.contains(word) {
                    return Err(AppError::UnknownWord(word.to_owned()));
                }
                grade(expected, word).map_err(AppError::GradingError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        state.num_guesses += 1;
        *should_delete =
            state.num_guesses > MAX_GUESSES || grade.iter().flatten().all(|x| *x == Grade::Correct);

        Ok(GuessResult { grade })
    }

    let mut should_delete = false;
    let id = guess.id;
    let ret = inner(&full_state, guess, &mut should_delete)?;

    if should_delete {
        full_state.mutable.write().unwrap().db.remove(&id);
    }

    Ok(Json(ret))
}

async fn root() -> &'static str {
    "Hello, World!"
}
