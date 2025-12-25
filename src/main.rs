use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex, RwLock},
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::cors::CorsLayer;
use ts_rs::TS;
use uuid::Uuid;

pub mod lastfm;

struct Config {
    lastfm_apikey: String,
    auth_callback_url: String,
}

#[derive(Clone)]
struct SharedState {
    mutable: Arc<RwLock<AppState>>,
    config: Arc<Config>,
    pool: SqlitePool,
    lastfm: Arc<lastfm::Client>,
}

#[derive(Default)]
struct AppState {
    db: HashMap<Uuid, Mutex<SessionState>>,
}

#[derive(Clone)]
struct SessionState {
    word: String,
    guesses: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("no such session")]
    NoSession,
    #[error("{0}")]
    GradingError(GradingError),
    #[error("something went wrong while contacting LastFM: {0}")]
    LastFmError(lastfm::Error),
    #[error("missing parameter {0}")]
    MissingParam(&'static str),
    #[error("user has no albums")]
    NoAlbums,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let status = match &self {
            AppError::NoSession => StatusCode::NOT_FOUND,
            AppError::NoAlbums | AppError::MissingParam(_) | AppError::GradingError(_) => {
                StatusCode::BAD_REQUEST
            }
            AppError::LastFmError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
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
    #[error("Wrong Length (expected {0}, have {1})")]
    WrongLength(usize, usize),
}

impl SessionState {
    fn new(word: String) -> Self {
        Self {
            word,
            guesses: Vec::new(),
        }
    }

    fn grade(&self, guess: &str) -> Result<Vec<Grade>, GradingError> {
        if self.word.len() != guess.len() {
            return Err(GradingError::WrongLength(self.word.len(), guess.len()));
        }
        let mut word: Vec<_> = guess.chars().map(Some).collect();
        let mut expected: Vec<_> = self.word.chars().map(Some).collect();

        let mut ret = vec![Grade::Incorrect; self.word.len()];

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
    };

    // ZMAC0-5khEKj-24kvmsnKdv1V_O2QqwX

    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/newgame", post(newgame))
        .route("/api/v1/guess", post(guess))
        .route("/api/v1/top-albums", get(get_top_albums))
        .route("/signin", get(signin))
        .route("/authenticate", get(authenticate))
        .layer(
            CorsLayer::new()
                .allow_headers(tower_http::cors::Any)
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn authenticate(State(state): State<SharedState>) -> Redirect {
    let api_key = &state.config.lastfm_apikey;
    let cb = &state.config.auth_callback_url;
    Redirect::to(&format!(
        "http://www.last.fm/api/auth/?api_key={api_key}&cb={cb}"
    ))
}

#[derive(Deserialize)]
struct SigninQuery {
    token: String,
}
async fn signin(State(state): State<SharedState>, Query(query): Query<SigninQuery>) -> String {
    let token = query.token;

    let session = state
        .lastfm
        .authenticate(&token)
        .await
        .expect("can authenticate");

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
    .expect("could insert");

    "success!".into()
}

async fn get_top_albums(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    state
        .lastfm
        .get_top_albums(query.get("user").ok_or(AppError::MissingParam("user"))?)
        .await
        .map_err(AppError::LastFmError)
        .map(Json)
}

#[derive(Serialize, TS)]
#[ts(export)]
struct NewGameResult {
    id: Uuid,
    len: usize,
}

async fn newgame(State(state): State<SharedState>) -> Result<Json<NewGameResult>, AppError> {
    let resp = state
        .lastfm
        .get_top_albums("hydehsmf")
        .await
        .map_err(AppError::LastFmError)?;
    let mut rng = rand::rng();
    let word = resp
        .albums
        .into_iter()
        .map(|x| x.name)
        .choose(&mut rng)
        .ok_or(AppError::NoAlbums)?;
    let word = word.to_lowercase().replace(' ', "");

    let id = Uuid::new_v4();
    let state = &mut state.mutable.write().unwrap();

    let len = word.len();
    state.db.insert(id, Mutex::new(SessionState::new(word)));
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
    grade: Vec<Grade>,
}

async fn guess(
    State(full_state): State<SharedState>,
    Json(guess): Json<GuessArgs>,
) -> Result<Json<GuessResult>, AppError> {
    let grade = {
        let state = &full_state.mutable.read().unwrap();
        let state = state.db.get(&guess.id).ok_or(AppError::NoSession)?;
        let state = state.lock().unwrap();
        state.grade(&guess.guess).map_err(AppError::GradingError)?
    };

    if grade.iter().all(|x| *x == Grade::Correct) {
        full_state.mutable.write().unwrap().db.remove(&guess.id);
    }

    Ok(Json(GuessResult { grade }))
}

async fn root() -> &'static str {
    "Hello, World!"
}
