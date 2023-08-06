use axum::{
    body::{boxed, Body, BoxBody},
    extract::{Form, State},
    http::{header::SET_COOKIE, HeaderMap, Request, Response, StatusCode, Uri},
    middleware::{from_fn, Next},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use blake3::hash;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    error::Error,
    fs::read_to_string,
    io::Write,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tower::util::ServiceExt;
use tower_http::services::ServeDir;

// TODO: show solved chals in /challenges, /profile (maybe /scoreboard)
// TODO: rewrite flag_submit, register_post and login_post endpoints to not use "success" var
// TODO: there should be a trigger to enable challenges endpoint when the CTF starts
// NOTE: branding change: templates. challenges: challenges.json

const AUTH_SECRET: &str = "CHANGE_ME!";
const ENABLE_STDOUT_EVENT_LOGS: bool = true;
static CHALLENGES: OnceLock<Vec<ChallengeCategory>> = OnceLock::new();
static TEMPLATE_CACHE: OnceLock<HashMap<&str, String>> = OnceLock::new();
static SCOREBOARD_CACHE: OnceLock<Mutex<String>> = OnceLock::new();

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const GOLD: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[00m";

fn initialize_scoreboard_cache(users: &BTreeSet<User>) {
    println!("Initializing scoreboard cache");
    if users.is_empty() {
        SCOREBOARD_CACHE
            .set(Mutex::new(String::from(
                "<article><h2 style=\"text-align: center;\">No users yet!</h2></article>",
            )))
            .unwrap();
    } else {
        SCOREBOARD_CACHE
            .set(Mutex::new(
                ScoreboardTemplate { users }.render_once().unwrap(),
            ))
            .unwrap();
    }
}

#[derive(Deserialize, Debug)]
struct ChallengeCategory {
    name: String,
    challenges: Vec<Challenge>,
}

#[derive(Deserialize, Debug)]
struct Challenge {
    id: u16,
    name: String,
    description: String,
    hint: Option<String>,
    points: u32,
    flag: String,
}

fn initialize_challenges() -> Result<(), Box<dyn Error>> {
    println!("Initializing challenges from config");
    let chals: Vec<ChallengeCategory> =
        serde_json::from_str(&read_to_string("./challenges.json")?)?;
    CHALLENGES.set(chals).unwrap();
    Ok(())
}

#[derive(TemplateOnce)]
#[template(path = "../templates/base.html", escape = false)]
struct BaseTemplate<'a> {
    head: &'a str,
    navbar: &'a str,
    body: &'a str,
}

#[derive(TemplateOnce)]
#[template(path = "../templates/info-box.html", escape = false)]
struct InfoBoxTemplate<'a> {
    success: bool,
    content: &'a str,
}
#[derive(TemplateOnce)]
#[template(path = "../templates/scoreboard.html", escape = false)]
struct ScoreboardTemplate<'a> {
    users: &'a BTreeSet<User>,
}

#[derive(TemplateOnce)]
#[template(path = "../templates/challenges.html", escape = false)]
struct ChallengesTemplate<'a> {
    challenges: &'a Vec<ChallengeCategory>,
    solves: &'a Vec<u16>,
}

#[derive(TemplateOnce)]
#[template(path = "../templates/profile.html", escape = false)]
struct ProfileTemplate<'a> {
    user: &'a User,
    // challenges: &'a Vec<Challenge>,
}

fn initialize_template_cache() -> Result<(), Box<dyn Error>> {
    println!("Initializing template cache");
    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("/", read_to_string("./templates/index.html")?);
    map.insert("/register", read_to_string("./templates/register.html")?);
    map.insert("/login", read_to_string("./templates/login.html")?);
    map.insert("navbar", read_to_string("./templates/navbar.html")?);
    map.insert(
        "navbar-logged",
        read_to_string("./templates/navbar-logged.html")?,
    );

    TEMPLATE_CACHE.set(map).unwrap();
    Ok(())
}

fn db_insert_user(database: Arc<Mutex<DB>>, user: User) -> Result<(), Box<dyn Error>> {
    database
        .lock()
        .unwrap()
        .set(user.username.to_owned(), user)?;
    Ok(())
}

async fn file_handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => get_static_file(uri_html).await,
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    match ServeDir::new("./static").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
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

async fn log_requests(req: Request<Body>, next: Next<Body>) -> impl IntoResponse {
    println!(
        "{GRAY}[{}] {RESET}Request: {GREEN}{} {BLUE}{}{RESET}",
        get_timestamp(),
        req.method(),
        req.uri()
    );

    next.run(req).await
}

fn get_navbar(logged: bool) -> &'static str {
    if logged {
        &TEMPLATE_CACHE.get().unwrap()["navbar-logged"]
    } else {
        &TEMPLATE_CACHE.get().unwrap()["navbar"]
    }
}

async fn root(headers: HeaderMap) -> Html<String> {
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &TEMPLATE_CACHE.get().unwrap()["/"],
        }
        .render_once()
        .unwrap(),
    )
}

async fn challenges(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Html<String> {
    let cookie = headers.get("cookie");
    let mut solves = Vec::new();
    if cookie.is_some() {
        let cookies = parse_cookie(headers.get("cookie").unwrap().to_str().unwrap());
        let username = get_cookie_value(&cookies, "username").unwrap();
        solves = state
            .database
            .lock()
            .unwrap()
            .get(username)
            .unwrap()
            .solves
            .clone();
    }
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &ChallengesTemplate {
                challenges: CHALLENGES.get().unwrap(),
                solves: &solves,
            }
            .render_once()
            .unwrap(),
        }
        .render_once()
        .unwrap(),
    )
}

async fn scoreboard(headers: HeaderMap) -> Html<String> {
    // TODO: paging? this would speedup this endpoint when lots of users (100 per page)
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &SCOREBOARD_CACHE.get().unwrap().lock().unwrap(),
        }
        .render_once()
        .unwrap(),
    )
}

async fn register(headers: HeaderMap) -> Html<String> {
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &TEMPLATE_CACHE.get().unwrap()["/register"],
        }
        .render_once()
        .unwrap(),
    )
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

async fn register_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(user): Form<UserRegister>,
) -> Html<String> {
    let mut body = InfoBoxTemplate {
        success: true,
        content: "Register successful",
    }
    .render_once()
    .unwrap();
    let mut success = true;

    // username must not already exist
    if state
        .database
        .lock()
        .unwrap()
        .db
        .get(&user.username)
        .is_some()
    {
        log_stdout(format!(
            "Register attempt {RED}failed{RESET} with: {BLUE}Username already registered{RESET} [username: {}] [email: {}]",
            user.username, user.email
        ));
        body = InfoBoxTemplate {
            success: false,
            content: "Username already registered!",
        }
        .render_once()
        .unwrap();
        success = false;
    }

    if success {
        // password and password_confirm should match
        if user.password != user.confirm_password {
            log_stdout(format!(
                "Register attempt {RED}failed{RESET} with: {BLUE}Passwords do not match{RESET} [username: {}] [email: {}]",
                user.username, user.email
            ));
            body = InfoBoxTemplate {
                success: false,
                content: "Passwords do not match!",
            }
            .render_once()
            .unwrap();
            success = false;
        }
    }

    if success {
        // password max length
        if user.password.len() < 4 || user.password.len() > 64 {
            log_stdout(format!(
                "Register attempt {RED}failed{RESET} with: {BLUE}Password wrong length{RESET} [username: {}] [email: {}]",
                user.username, user.email
            ));
            body = InfoBoxTemplate {
                success: false,
                content: "Password length should be in the range: 4-64",
            }
            .render_once()
            .unwrap();
            success = false;
        }
    }

    // maybe: ensure email is in a email format

    if success {
        log_stdout(format!(
            "Register {GOLD}success{RESET} [username: {}] [email: {}]",
            user.username, user.email
        ));
        let u = User {
            id: 0,
            username: user.username,
            email: user.email,
            // hashing here instead of in the db_insert_user
            password: hash(user.password.as_bytes()).to_hex().to_string(),
            score: 0,
            solves: Vec::new(),
        };
        db_insert_user(state.database.clone(), u).unwrap();
    }

    body.push_str(&TEMPLATE_CACHE.get().unwrap()["/register"]);

    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &body,
        }
        .render_once()
        .unwrap(),
    )
}

async fn login(headers: HeaderMap) -> Html<String> {
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &TEMPLATE_CACHE.get().unwrap()["/login"],
        }
        .render_once()
        .unwrap(),
    )
}

#[derive(Debug, Deserialize)]
struct UserLogin {
    username: String,
    password: String,
}

async fn login_post(
    State(state): State<Arc<AppState>>,
    Form(user): Form<UserLogin>,
) -> impl IntoResponse {
    let mut success = true;
    let mut body = InfoBoxTemplate {
        success: true,
        content: "Login successful",
    }
    .render_once()
    .unwrap();
    let pass_hash = hash(user.password.as_bytes()).to_hex().to_string();

    let mut user_found = true;
    let expected_hash: String = match state.database.lock().unwrap().db.get(&user.username) {
        Some(ok) => ok.password.to_owned(),
        None => {
            user_found = false;
            success = false;
            log_stdout(format!(
                "Login attempt {RED}failed{RESET} with: {BLUE}User not found{RESET} [username: {}]",
                user.username
            ));
            body = InfoBoxTemplate {
                success: false,
                content: "User not found!",
            }
            .render_once()
            .unwrap();
            "".to_string()
        }
    };

    if expected_hash != pass_hash && user_found {
        log_stdout(format!(
            "Login attempt {RED}failed{RESET} with: {BLUE}Wrong password{RESET} [username: {}]",
            user.username
        ));
        body = InfoBoxTemplate {
            success: false,
            content: "Wrong password!",
        }
        .render_once()
        .unwrap();
        success = false;
    }

    let mut out_headers = HeaderMap::new();
    if success {
        log_stdout(format!(
            "Login attempt {GOLD}success{RESET} [username: {}]",
            user.username
        ));
        let auth_key = hash((AUTH_SECRET.to_owned() + &pass_hash).as_bytes())
            .to_hex()
            .to_string();
        out_headers.insert(
            SET_COOKIE,
            format!("username={};", user.username).parse().unwrap(),
        );
        out_headers.append(SET_COOKIE, format!("auth_key={auth_key};").parse().unwrap());
    }

    body.push_str(&TEMPLATE_CACHE.get().unwrap()["/"]);

    (
        out_headers,
        Html(
            BaseTemplate {
                head: "",
                navbar: get_navbar(success),
                body: &body,
            }
            .render_once()
            .unwrap(),
        ),
    )
}

async fn profile(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Html<String> {
    let cookies = parse_cookie(headers.get("cookie").unwrap().to_str().unwrap());
    let username = get_cookie_value(&cookies, "username").unwrap();
    // TODO: render solved challenges and scoreboard position
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &ProfileTemplate {
                user: state.database.lock().unwrap().get(username).unwrap(),
                // challenges: &CHALLENGES.get().unwrap(),
            }
            .render_once()
            .unwrap(),
        }
        .render_once()
        .unwrap(),
    )
}

#[derive(Debug, Deserialize)]
struct FlagSubmition {
    challenge_id: u16,
    flag: String,
}

fn get_cookie_value<'a>(cookies: &Vec<(&str, &'a str)>, cookie_name: &'a str) -> Option<&'a str> {
    for cookie in cookies {
        if cookie.0 == cookie_name {
            return Some(cookie.1);
        }
    }
    None
}

fn parse_cookie(cookie: &str) -> Vec<(&str, &str)> {
    cookie
        .split(';')
        .map(|e| {
            let spl = e.split('=').collect::<Vec<&str>>();
            (spl[0].trim(), spl[1].trim())
        })
        .collect()
}

async fn flag_submit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(submition): Form<FlagSubmition>,
) -> Html<String> {
    let mut body = InfoBoxTemplate {
        success: true,
        content: "Flag accepted",
    }
    .render_once()
    .unwrap();
    let mut success = true;

    let cookie = headers.get("cookie");

    if cookie.is_none() {
        body = InfoBoxTemplate {
            success: false,
            content: "You need to be logged in to submit flags!",
        }
        .render_once()
        .unwrap();
        // HACK
        body.push_str(
            &ChallengesTemplate {
                challenges: CHALLENGES.get().unwrap(),
                solves: &Vec::new(),
            }
            .render_once()
            .unwrap(),
        );
        return Html(
            BaseTemplate {
                head: "",
                navbar: get_navbar(headers.get("cookie").is_some()),
                body: &body,
            }
            .render_once()
            .unwrap(),
        );
    }

    // maybe: validate user exists (changing cookie to a user which doesnt exist will crash the app)

    let mut conn = state.database.lock().unwrap();
    let cookies = parse_cookie(cookie.unwrap().to_str().unwrap());
    let username = get_cookie_value(&cookies, "username").unwrap();
    let auth_key = get_cookie_value(&cookies, "auth_key").unwrap();

    // check if user already solved a challenge
    if conn
        .db
        .get(username)
        .unwrap()
        .solves
        .contains(&submition.challenge_id)
    {
        body = InfoBoxTemplate {
            success: false,
            content: "You have already solved this challenge!",
        }
        .render_once()
        .unwrap();
        success = false;
    }

    // check if flag is correct
    let mut chal_points = 0;
    if success {
        for chal_cat in CHALLENGES.get().unwrap() {
            for chal in &chal_cat.challenges {
                if chal.id == submition.challenge_id {
                    chal_points = chal.points;
                    if chal.flag != submition.flag {
                        log_stdout(format!(
                        "Flag submit {RED}failed{RESET} with: Wrong flag: {submition:?} [username: {username}]"
                    ));
                        body = InfoBoxTemplate {
                            success: false,
                            content: "Wrong flag!",
                        }
                        .render_once()
                        .unwrap();
                        success = false;
                    }
                    break;
                }
            }
        }
    }

    // check authentication
    if success {
        let pass_hash = &conn.get(username).unwrap().password;
        let expected_auth_key = hash((AUTH_SECRET.to_owned() + &pass_hash).as_bytes()).to_string();
        if expected_auth_key != auth_key {
            log_stdout(format!(
                "Flag submit {RED}failed{RESET} with: Authentication failed: {submition:?} [username: {username}]"
            ));
            body = InfoBoxTemplate {
                success: false,
                content: "Authentication failed!",
            }
            .render_once()
            .unwrap();
            success = false;
        }
    }

    // if checks passed, add the points and mark as solved
    if success {
        log_stdout(format!(
            "Flag submit {GOLD}success{RESET}: {submition:?} [username: {username}]"
        ));
        let mut user = conn.get(username).unwrap().clone();
        user.solves.push(submition.challenge_id);
        user.score += chal_points;
        conn.set(username.to_string(), user).unwrap();
    }

    body.push_str(
        &ChallengesTemplate {
            challenges: CHALLENGES.get().unwrap(),
            solves: &conn.get(username).unwrap().solves.clone(),
        }
        .render_once()
        .unwrap(),
    );
    Html(
        BaseTemplate {
            head: "",
            navbar: get_navbar(headers.get("cookie").is_some()),
            body: &body,
        }
        .render_once()
        .unwrap(),
    )
}

async fn logout(headers: HeaderMap) -> impl IntoResponse {
    let mut body = InfoBoxTemplate {
        success: true,
        content: "Logged out",
    }
    .render_once()
    .unwrap();
    body.push_str(&TEMPLATE_CACHE.get().unwrap()["/"]);
    let cookies = parse_cookie(headers.get("cookie").unwrap().to_str().unwrap());
    let username = get_cookie_value(&cookies, "username").unwrap();
    log_stdout(format!("User {BLUE}{username} {RESET}logged out"));
    let mut out_headers = HeaderMap::new();
    out_headers.insert("Clear-Site-Data", "\"cookies\"".parse().unwrap());
    (
        out_headers,
        Html(
            BaseTemplate {
                head: "",
                navbar: get_navbar(false),
                body: &body,
            }
            .render_once()
            .unwrap(),
        ),
    )
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

impl Ord for User {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.score.cmp(&self.score) {
            Ordering::Equal => self.username.cmp(&other.username),
            ordering => ordering,
        }
    }
}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for User {
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
    set: BTreeSet<User>,
}

impl DBInner {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            set: BTreeSet::new(),
        }
    }

    fn get(&self, username: &str) -> Option<&User> {
        self.map.get(username)
    }

    fn set(&mut self, username: String, user: User) {
        // Check if the username exists in the HashMap
        if let Some(existing_user) = self.map.get_mut(&username) {
            // Remove the existing user from the BTreeSet
            self.set.remove(existing_user);

            // Update the existing user
            *existing_user = user;

            // Insert the updated user back into the BTreeSet
            self.set.insert(existing_user.clone());
        } else {
            // Insert the new user into the HashMap
            self.map.insert(username.clone(), user.clone());

            // Insert the new user into the BTreeSet
            self.set.insert(user);
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
                db: DBInner::new(),
                filename: filename.to_string(),
            }
        }
    }

    fn set(&mut self, k: String, v: User) -> Result<(), Box<dyn Error>> {
        self.db.set(k, v);
        // save serialized to disk
        let serialized = serde_json::to_string(&self.db)?;
        let mut fh = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.filename)?;
        fh.write_all(serialized.as_bytes())?;
        // update scoreboard cache
        *SCOREBOARD_CACHE.get().unwrap().lock().unwrap() = ScoreboardTemplate {
            users: &self.db.set,
        }
        .render_once()
        .unwrap();
        Ok(())
    }

    fn get(&self, username: &str) -> Option<&User> {
        self.db.get(username)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bind_addr = "0.0.0.0:3000";

    let database = Arc::new(Mutex::new(DB::new("./database.db")));

    let state_routes = Router::new()
        .route("/scoreboard", get(scoreboard))
        .route("/register", post(register_post))
        .route("/login", post(login_post))
        .route("/profile", get(profile))
        .route("/flag_submit", post(flag_submit))
        .route("/challenges", get(challenges))
        .with_state(Arc::new(AppState {
            database: database.clone(),
        }));

    let plain_routes = Router::new()
        .route("/", get(root))
        .route("/logout", get(logout))
        .route("/register", get(register))
        .route("/login", get(login));

    let app = Router::new()
        .nest_service("/static", get(file_handler))
        .merge(plain_routes)
        .merge(state_routes)
        .layer(from_fn(log_requests)); // uncomment for request logging. comment for better perf

    println!("Starting the app on: {GOLD}{bind_addr}{RESET}");

    initialize_challenges().unwrap_or_else(|err| {
        println!("Error: {err}");
        std::process::exit(1);
    });
    initialize_template_cache().unwrap_or_else(|err| {
        println!("Error: {err}");
        std::process::exit(1);
    });
    initialize_scoreboard_cache(&database.lock().unwrap().db.set);

    axum::Server::bind(&bind_addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
