use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Method, Request},
    middleware::{from_fn, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use blake3::hash;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    error::Error,
    fs::read_to_string,
    io::Write,
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::{Any, CorsLayer};

// TODO: show solved chals in /challenges, /profile (maybe /scoreboard)
// TODO: use rust-argon2 instead of blake3 for password hashing (AUTH_SECRET as salt?)
// TODO: dynamic scoring system
// TODO: change_pass, ?admin panel?, your position on the scoreboard page (above the table)
// TODO: hostctf frontend render "your position"
//       (backend should return an array with two things: 1.logged_user_stats 2.all_users_or_first_X_users)

const CTF_STARTED: bool = true;
const AUTH_SECRET: &str = "CHANGE_ME!"; // NOTE: use tower auth layer instead?
const ENABLE_STDOUT_EVENT_LOGS: bool = true;

static CHALLENGES: OnceLock<String> = OnceLock::new();
static CHALLENGES_JSON: OnceLock<Vec<ChallengeCategory>> = OnceLock::new();
static SCOREBOARD_CACHE: Mutex<String> = Mutex::new(String::new());

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const GOLD: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[00m";

#[derive(Deserialize, Debug, Serialize)]
struct Challenge {
    id: u16,
    name: String,
    description: String,
    hint: Option<String>,
    points: u32,
    flag: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct ChallengeCategory {
    name: String,
    challenges: Vec<Challenge>,
}

#[derive(Deserialize, Debug, Serialize)]
struct ChallengeNoFlag {
    id: u16,
    name: String,
    description: String,
    hint: Option<String>,
    points: u32,
}

#[derive(Deserialize, Debug, Serialize)]
struct ChallengeNoFlagCategory {
    name: String,
    challenges: Vec<ChallengeNoFlag>,
}

fn initialize_challenges() -> Result<(), Box<dyn Error>> {
    println!("Initializing challenges from config");
    let challs_str = read_to_string("./challenges.json")?;
    let challs: Vec<ChallengeCategory> = serde_json::from_str(&challs_str)?;
    CHALLENGES_JSON.set(challs).unwrap();
    let challs_no_flag: Vec<ChallengeNoFlagCategory> = serde_json::from_str(&challs_str)?;
    CHALLENGES
        .set(serde_json::to_string(&challs_no_flag)?)
        .unwrap();
    Ok(())
}

fn initialize_scoreboard_cache(users: &BTreeSet<UserScoreboard>) -> Result<(), Box<dyn Error>> {
    println!("Initializing scoreboard cache");
    *SCOREBOARD_CACHE.lock().unwrap() = serde_json::to_string(users)?;
    Ok(())
}

fn get_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let in_secs = since_the_epoch.as_secs();
    let secs_of_day = in_secs % (24 * 60 * 60);
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;

    let days_since_epoch = in_secs / (24 * 60 * 60);
    let year = 1970 + days_since_epoch / 365;
    let month = (days_since_epoch % 365) / 30 + 1;
    let day = days_since_epoch % 30 + 1;

    format!(
        "{:02}-{:02}-{} {:02}:{:02}:{:02}",
        day, month, year, hours, minutes, seconds
    )
}

async fn log_requests(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next<Body>,
) -> impl IntoResponse {
    println!(
        "{GRAY}[{}] {RESET}[{}]: {GREEN}{} {BLUE}{}{RESET}",
        get_timestamp(),
        addr.ip(),
        req.method(),
        req.uri()
    );

    next.run(req).await
}

fn get_auth_key(pass_hash: &str) -> String {
    hash((AUTH_SECRET.to_owned() + pass_hash).as_bytes())
        .to_hex()
        .to_string()
}

async fn flag_submit(
    State(state): State<Arc<AppState>>,
    Json(submition): Json<FlagSubmition>,
) -> &'static str {
    // need to be logged in
    if submition.username.is_empty() || submition.auth_key.is_empty() {
        return "{\"status\":\"FAIL\",\"cause\":\"You need to be logged in to submit flags\"}";
    }

    let mut db = state.database.lock().unwrap();
    let db_user = match db.get(&submition.username) {
        Some(ok) => ok,
        None => {
            return "{\"status\":\"FAIL\",\"cause\":\"User does not exist\"}";
        }
    };

    // check authentication
    if get_auth_key(&db_user.password) != submition.auth_key {
        log_stdout(format!(
            "Flag submit attempt {RED}failed{RESET} with: {BLUE}Authentication failed: {submition:?}{RESET} [username: {}]",
            submition.username
        ));
        return "{\"status\":\"FAIL\",\"cause\":\"Authentication failed\"}";
    }

    // already solved the challenge
    if db_user.solves.contains(&submition.challenge_id) {
        return "{\"status\":\"FAIL\",\"cause\":\"You have already solved this challenge\"}";
    }

    let mut points: Option<u32> = None;
    for chal_cat in CHALLENGES_JSON.get().unwrap() {
        for chall in &chal_cat.challenges {
            if chall.id == submition.challenge_id {
                points = Some(chall.points);
                if chall.flag != submition.flag {
                    log_stdout(format!(
                        "Flag submit attempt {RED}failed{RESET} with: {BLUE}Wrong flag ({}):{}{RESET} [username: {}]",
                        submition.challenge_id, submition.flag, submition.username
                    ));
                    return "{\"status\":\"FAIL\",\"cause\":\"Wrong flag\"}";
                }
            }
        }
    }

    if points.is_none() {
        return "{\"status\":\"FAIL\",\"cause\":\"Challenge doesnt exist\"}";
    }

    log_stdout(format!(
        "Flag submit attempt {GOLD}success{RESET}: {BLUE}({}):{}{RESET} [username: {}]",
        submition.challenge_id, submition.flag, submition.username
    ));

    let mut u = db_user.clone();
    u.solves.push(submition.challenge_id);
    u.score += points.unwrap();
    db.set(submition.username, u).unwrap();

    "{\"status\":\"OK\"}"
}

async fn challenges() -> &'static str {
    if CTF_STARTED {
        CHALLENGES.get().unwrap()
    } else {
        "{}"
    }
}

async fn scoreboard() -> String {
    // TODO: return only first X records for better performance
    SCOREBOARD_CACHE.lock().unwrap().to_string()
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(user): Json<UserRegister>,
) -> &'static str {
    // all fields must not by empty (except confirm_password as that is checked later anyway)
    if user.username.is_empty() || user.password.is_empty() || user.email.is_empty() {
        return "{\"status\":\"FAIL\",\"cause\":\"Fields must not be empty\"}";
    }

    // username already registered
    if state.database.lock().unwrap().get(&user.username).is_some() {
        log_stdout(format!(
            "Register attempt {RED}failed{RESET} with: {BLUE}Username already registered{RESET} [username: {}] [email: {}]",
            user.username, user.email
        ));
        return "{\"status\":\"FAIL\",\"cause\":\"Username already registered\"}";
    }

    // passwords do not match
    if user.password != user.confirm_password {
        log_stdout(format!(
            "Register attempt {RED}failed{RESET} with: {BLUE}Passwords do not match{RESET} [username: {}] [email: {}]",
            user.username, user.email
        ));
        return "{\"status\":\"FAIL\",\"cause\":\"Passwords do not match\"}";
    }

    // password wrong length
    if user.password.len() < 4 || user.password.len() > 64 {
        log_stdout(format!(
            "Register attempt {RED}failed{RESET} with: {BLUE}Passwords wrong length{RESET} [username: {}] [email: {}]",
            user.username, user.email
        ));
        return "{\"status\":\"FAIL\",\"cause\":\"Password length should be in the range: 4-64\"}";
    }

    log_stdout(format!(
        "Register attempt {GOLD}success{RESET} [username: {}] [email: {}]",
        user.username, user.email
    ));

    // actually register to DB
    let u = User {
        id: 0,
        username: user.username,
        email: user.email,
        password: hash(user.password.as_bytes()).to_hex().to_string(),
        score: 0,
        solves: Vec::new(),
    };
    state
        .database
        .lock()
        .unwrap()
        .set(u.username.to_owned(), u)
        .unwrap();

    "{\"status\":\"OK\"}"
}

#[derive(Debug, Deserialize)]
struct UserRegister {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
}

fn log_stdout(mes: String) {
    if ENABLE_STDOUT_EVENT_LOGS {
        println!("{GRAY}[{}]{RESET} {mes}", get_timestamp());
    }
}

async fn login(State(state): State<Arc<AppState>>, Json(user): Json<UserLogin>) -> String {
    let db = state.database.lock().unwrap();
    let db_user: &User = match db.get(&user.username) {
        Some(ok) => ok,
        None => {
            log_stdout(format!(
                "Login attempt {RED}failed{RESET} with: {BLUE}User not found{RESET} [username: {}]",
                user.username
            ));
            return "{\"status\":\"FAIL\",\"cause\":\"User not found\"}".to_string();
        }
    };

    // correct password
    if hash(user.password.as_bytes()).to_hex().to_string() != db_user.password {
        log_stdout(format!(
            "Login attempt {RED}failed{RESET} with: {BLUE}Wrong password{RESET} [username: {}]",
            user.username
        ));
        return "{\"status\":\"FAIL\",\"cause\":\"Wrong password\"}".to_string();
    }

    log_stdout(format!(
        "Login attempt {GOLD}success{RESET} [username: {}]",
        user.username
    ));

    let auth_key = get_auth_key(&db_user.password);
    let solved_chals = serde_json::to_string(&db_user.solves).unwrap();

    format!("{{\"status\":\"OK\",\"auth_key\":\"{auth_key}\",\"solved_chals\":{solved_chals}}}")
}

#[derive(Debug, Deserialize)]
struct UserLogin {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct ProfileRequest {
    username: String,
}

async fn profile(State(state): State<Arc<AppState>>, Json(user): Json<ProfileRequest>) -> String {
    let db = state.database.lock().unwrap();
    let db_user = match db.get(&user.username) {
        Some(ok) => ok,
        None => {
            return "{\"status\":\"FAIL\",\"cause\":\"User doesnt exist\"}".to_string();
        }
    };
    format!("{{\"status\":\"OK\",\"score\":{}}}", db_user.score)
}

#[derive(Debug, Deserialize)]
struct FlagSubmition {
    username: String,
    auth_key: String,
    challenge_id: u16,
    flag: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq)]
struct User {
    id: u32,
    username: String,
    email: String,
    password: String,
    score: u32,
    solves: Vec<u16>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.username == other.username
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq)]
struct UserScoreboard {
    username: String,
    score: u32,
}

impl Ord for UserScoreboard {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.score.cmp(&self.score) {
            Ordering::Equal => self.username.cmp(&other.username),
            ordering => ordering,
        }
    }
}

impl PartialOrd for UserScoreboard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for UserScoreboard {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.username == other.username
    }
}

#[derive(Debug)]
struct AppState {
    database: Arc<Mutex<DB>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DBInner {
    map: HashMap<String, User>,
    set: BTreeSet<UserScoreboard>,
}

impl DBInner {
    fn _new() -> Self {
        Self {
            map: HashMap::new(),
            set: BTreeSet::new(),
        }
    }

    fn _get(&self, username: &str) -> Option<&User> {
        self.map.get(username)
    }

    fn _set(&mut self, username: String, user: User) {
        // Check if the username exists in the HashMap
        if let Some(existing_user) = self.map.get_mut(&username) {
            // Existing user object in the set
            let mut u = UserScoreboard {
                username: existing_user.username.clone(),
                score: existing_user.score,
            };

            // Remove the existing user from the BTreeSet
            self.set.remove(&u);

            // Update the existing user in the map
            *existing_user = user;

            // Updated set user
            u = UserScoreboard {
                username: existing_user.username.clone(),
                score: existing_user.score,
            };

            // Insert the updated user back into the BTreeSet
            self.set.insert(u);
        } else {
            // Insert the new user into the HashMap
            self.map.insert(username.clone(), user.clone());

            // New user for the set
            let u = UserScoreboard {
                username: user.username,
                score: user.score,
            };

            // Insert the new user into the BTreeSet
            self.set.insert(u);
        }
    }
}

#[derive(Debug)]
struct DB {
    db: DBInner,
    filename: String,
}

impl DB {
    fn new(filename: &str) -> Self {
        if Path::new(filename).exists() {
            Self {
                db: serde_json::from_str(&read_to_string(filename).unwrap()).unwrap(),
                filename: filename.to_string(),
            }
        } else {
            Self {
                db: DBInner::_new(),
                filename: filename.to_string(),
            }
        }
    }

    fn set(&mut self, k: String, v: User) -> Result<(), Box<dyn Error>> {
        self.db._set(k, v);
        // save serialized to disk
        let serialized = serde_json::to_string(&self.db)?;
        let mut fh = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.filename)?;
        fh.write_all(serialized.as_bytes())?;
        // update scoreboard cache
        *SCOREBOARD_CACHE.lock().unwrap() = serde_json::to_string(&self.db.set)?;
        Ok(())
    }

    fn get(&self, username: &str) -> Option<&User> {
        self.db._get(username)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bind_addr = "0.0.0.0:3000";

    let database = Arc::new(Mutex::new(DB::new("./database.db")));

    let routes = Router::new()
        .route("/scoreboard", get(scoreboard))
        .route("/challenges", get(challenges));

    let db_routes = Router::new()
        .route("/flag_submit", post(flag_submit))
        .route("/profile", post(profile))
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(Arc::new(AppState {
            database: database.clone(),
        }));

    // TODO: change_pass POST endpoint
    let app = Router::new()
        .merge(routes)
        .merge(db_routes)
        .layer(
            CorsLayer::new() // I would love to get rid of this (for a substantial performance boost) but the browser won't let me do that
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any)
                .allow_origin(Any),
        )
        .layer(from_fn(log_requests)); // uncomment for request logging. comment for better perf

    initialize_challenges()?;
    initialize_scoreboard_cache(&database.lock().unwrap().db.set)?;

    println!("Starting {GOLD}backend{RESET} on: {GOLD}{bind_addr}{RESET}");

    axum::Server::bind(&bind_addr.parse()?)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
