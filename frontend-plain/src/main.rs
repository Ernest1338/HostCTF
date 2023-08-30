use axum::{extract::Path, http::StatusCode, response::Html, routing::get, Router};
use minijinja::render;
use std::{collections::HashMap, error::Error, fs::read_to_string, sync::OnceLock};

const BACKEND_ADDR: &str = "http://localhost:3000";

static FRONTEND_CACHE: OnceLock<HashMap<&str, String>> = OnceLock::new();
static BASE_TEMPLATE: OnceLock<String> = OnceLock::new();

fn initialize_base_template() -> Result<(), Box<dyn Error>> {
    println!("Initializing base template");
    BASE_TEMPLATE
        .set(read_to_string("templates/base.html")?)
        .unwrap();
    Ok(())
}

fn load_template(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = read_to_string(path)?;
    Ok(render!(BASE_TEMPLATE.get().unwrap(), body => body))
}

fn initialize_frontend_cache() -> Result<(), Box<dyn Error>> {
    println!("Initializing main cache");
    let mut cache: HashMap<&str, String> = HashMap::new();
    cache.insert("", load_template("templates/index.html")?);
    cache.insert("register", load_template("templates/register.html")?);
    cache.insert("login", load_template("templates/login.html")?);
    cache.insert("challenges", load_template("templates/challenges.html")?);
    cache.insert("scoreboard", load_template("templates/scoreboard.html")?);
    cache.insert(
        "script",
        render!(
            &read_to_string("templates/script.js")?,
            backend_addr => BACKEND_ADDR
        ),
    );
    cache.insert("logout", load_template("templates/index.html")?);
    cache.insert("profile", load_template("templates/profile.html")?);
    FRONTEND_CACHE.set(cache).unwrap();
    Ok(())
}

async fn serve_static_content(
    Path(path): Path<String>,
) -> Result<Html<&'static str>, (StatusCode, &'static str)> {
    if let Some(content) = FRONTEND_CACHE.get().unwrap().get(path.as_str()) {
        Ok(Html(content))
    } else {
        Err((StatusCode::NOT_FOUND, "404 Not Found"))
    }
}

async fn root() -> Html<&'static str> {
    Html(&FRONTEND_CACHE.get().unwrap()[""])
}

async fn script() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Cache-Control", "public, max-age=31536000")],
        FRONTEND_CACHE.get().unwrap().get("script").unwrap(),
    )
}

#[tokio::main]
async fn main() {
    if let Err(e) = initialize_base_template() {
        eprintln!("Error initializing base template: {e}");
        return;
    };
    if let Err(e) = initialize_frontend_cache() {
        eprintln!("Error initializing frontend cache: {e}");
        return;
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/script", get(script))
        .route("/:path", get(serve_static_content));

    let bind_addr = "0.0.0.0:8080";
    println!("Starting \x1b[33mfrontend\x1b[00m on: \x1b[33m{bind_addr}\x1b[00m");
    // NOTE: maybe: implement logging like the backend has?

    axum::Server::bind(&bind_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
