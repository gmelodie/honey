//! Dashboard HTTP server for the precomputed stats and wordlists.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;

use crate::gz_path;
use serde_json::{Value, json};

const TEMPLATE: &str = include_str!("../web/templates/index.html");
const STYLE_CSS: &[u8] = include_bytes!("../web/static/css/style.css");
const APP_JS: &[u8] = include_bytes!("../web/static/js/app.js");
const FAVICON: &[u8] = include_bytes!("../web/static/favicon.svg");

const STATS_WINDOWS: [&str; 6] = ["1h", "6h", "24h", "7d", "30d", "all"];

/// `(url param, directory, look-back days)`.
const PERIODS: [(&str, &str, Option<i64>); 4] = [
    ("daily", "daily", Some(1)),
    ("weekly", "weekly", Some(7)),
    ("monthly", "monthly", Some(30)),
    ("all", "alltime", None),
];

/// `(url param, file name)`, in response order.
const WORDLISTS: [(&str, &str); 8] = [
    ("usernames", "usernames.txt"),
    ("passwords", "passwords.txt"),
    ("pairs", "passwords_usernames.txt"),
    ("novel_passwords", "novel_passwords.txt"),
    ("trending_passwords", "trending_passwords.txt"),
    ("dying_passwords", "dying_passwords.txt"),
    ("hashcat_rules", "hashcat.rule"),
    ("john_rules", "john.rule"),
];

pub struct Config {
    pub stats_dir: PathBuf,
    pub wordlist_dir: PathBuf,
    pub recaptcha_site_key: String,
    pub recaptcha_secret: String,
    pub recaptcha_min_score: f64,
    pub recaptcha_verify_url: String,
}

struct AppState {
    cfg: Config,
    index_html: String,
    http: reqwest::Client,
}

type Shared = State<Arc<AppState>>;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// JSON that is safe inside a `<script>` element.
fn script_json(s: &str) -> String {
    Value::from(s)
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\'', "\\u0027")
}

pub fn render_index(site_key: &str) -> String {
    let render = if site_key.is_empty() {
        "explicit"
    } else {
        site_key
    };
    TEMPLATE
        .replace("{{RECAPTCHA_RENDER}}", &html_escape(render))
        .replace("{{RECAPTCHA_SITE_KEY_JSON}}", &script_json(site_key))
}

pub fn router(cfg: Config) -> Router {
    let state = AppState {
        index_html: render_index(&cfg.recaptcha_site_key),
        cfg,
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("TLS backend initialises"),
    };
    Router::new()
        .route("/", get(index))
        .route("/static/{*path}", get(static_file))
        .route("/api/verify-captcha", post(verify_captcha))
        .route("/api/stats", get(stats))
        .route("/api/wordlist", get(wordlist_meta))
        .route("/api/wordlist/{wtype}/download", get(wordlist_download))
        .fallback(not_found)
        .with_state(Arc::new(state))
}

fn error(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({"error": msg}))).into_response()
}

async fn index(State(st): Shared) -> Html<String> {
    Html(st.index_html.clone())
}

async fn static_file(UrlPath(path): UrlPath<String>) -> Response {
    let (mime, body) = match path.as_str() {
        "css/style.css" => ("text/css; charset=utf-8", STYLE_CSS),
        "js/app.js" => ("text/javascript; charset=utf-8", APP_JS),
        "favicon.svg" => ("image/svg+xml", FAVICON),
        _ => return error(StatusCode::NOT_FOUND, "not found"),
    };
    ([(header::CONTENT_TYPE, mime)], body).into_response()
}

async fn not_found(uri: Uri) -> Response {
    if uri.path().starts_with("/api/") {
        return error(StatusCode::NOT_FOUND, "not found");
    }
    (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
}

async fn verify_captcha(State(st): Shared, body: Bytes) -> Response {
    if st.cfg.recaptcha_secret.is_empty() {
        return Json(json!({"success": true})).into_response();
    }

    let req: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let token = req["token"].as_str().unwrap_or("");
    if token.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"success": false}))).into_response();
    }

    let verdict = st
        .http
        .post(&st.cfg.recaptcha_verify_url)
        .form(&[
            ("secret", st.cfg.recaptcha_secret.as_str()),
            ("response", token),
        ])
        .send()
        .await
        .and_then(reqwest::Response::error_for_status);
    let body: Value = match verdict {
        Ok(resp) => resp.json().await.unwrap_or(Value::Null),
        Err(e) => {
            eprintln!("reCAPTCHA verify failed: {e}");
            return (StatusCode::BAD_GATEWAY, Json(json!({"success": false}))).into_response();
        }
    };

    let ok = body["success"].as_bool().unwrap_or(false)
        && body["score"].as_f64().unwrap_or(0.0) >= st.cfg.recaptcha_min_score;
    Json(json!({"success": ok})).into_response()
}

async fn stats(State(st): Shared, Query(q): Query<HashMap<String, String>>) -> Response {
    let window = q
        .get("window")
        .map(String::as_str)
        .filter(|w| STATS_WINDOWS.contains(w))
        .unwrap_or("6h");

    match tokio::fs::read(st.cfg.stats_dir.join(format!("{window}.json"))).await {
        Ok(data) => ([(header::CONTENT_TYPE, "application/json")], data).into_response(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => error(
            StatusCode::SERVICE_UNAVAILABLE,
            "stats not yet generated, please wait",
        ),
        Err(e) => {
            eprintln!("read stats {window}: {e}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "cannot read stats")
        }
    }
}

fn period_of(q: &HashMap<String, String>) -> (&'static str, &'static str, Option<i64>) {
    let want = q.get("period").map_or("all", String::as_str);
    PERIODS
        .into_iter()
        .find(|p| p.0 == want)
        .unwrap_or(PERIODS[3])
}

pub fn fmt_bytes(n: u64) -> String {
    let mut n = n as f64;
    for unit in ["B", "KB", "MB"] {
        if n < 1024.0 {
            return format!("{n:.0} {unit}");
        }
        n /= 1024.0;
    }
    format!("{n:.1} GB")
}

fn is_rule(wtype: &str) -> bool {
    wtype.ends_with("_rules")
}

fn is_rule_line(line: &str) -> bool {
    !line.is_empty() && !line.starts_with('#') && !line.starts_with('[')
}

/// Returns `(line count, first 5 preview lines)`.
fn scan(path: &Path, rule: bool) -> std::io::Result<(usize, Vec<String>)> {
    let mut count = 0;
    let mut preview = Vec::new();
    for line in BufReader::new(File::open(path)?).split(b'\n') {
        let line = String::from_utf8_lossy(&line?).into_owned();
        count += 1;
        if preview.len() < 5 && (!rule || is_rule_line(&line)) {
            preview.push(line);
        }
    }
    Ok((count, preview))
}

fn wordlist_info(path: &Path, wtype: &str, oldest: Option<&str>, newest: &str) -> Value {
    let not_ready = json!({"ready": false, "oldest": null, "newest": null});
    let Ok(meta) = std::fs::metadata(path) else {
        return not_ready;
    };
    let Ok((count, preview)) = scan(path, is_rule(wtype)) else {
        return not_ready;
    };

    let size = meta.len();
    let rule = is_rule(wtype);
    json!({
        "ready": true,
        "total": count,
        "oldest": if rule { None } else { oldest },
        "newest": if rule { None } else { Some(newest) },
        "size": fmt_bytes(size),
        "gz_size": std::fs::metadata(gz_path(path)).ok().map(|m| fmt_bytes(m.len())),
        "preview": preview,
    })
}

async fn wordlist_meta(State(st): Shared, Query(q): Query<HashMap<String, String>>) -> Response {
    let (period, dir, days) = period_of(&q);
    let dir = st.cfg.wordlist_dir.join(dir);
    let now = Utc::now();
    let oldest = days.map(|d| crate::iso(now - chrono::Duration::days(d)));
    let newest = crate::iso(now);

    let body = tokio::task::spawn_blocking(move || {
        let mut out = serde_json::Map::new();
        out.insert("period".into(), json!(period));
        for (wtype, file) in WORDLISTS {
            out.insert(
                wtype.into(),
                wordlist_info(&dir.join(file), wtype, oldest.as_deref(), &newest),
            );
        }
        Value::Object(out)
    })
    .await;

    match body {
        Ok(v) => Json(v).into_response(),
        Err(e) => {
            eprintln!("wordlist meta: {e}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "cannot read wordlists")
        }
    }
}

async fn wordlist_download(
    State(st): Shared,
    UrlPath(wtype): UrlPath<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Some((wtype, file)) = WORDLISTS.into_iter().find(|w| w.0 == wtype) else {
        return error(StatusCode::BAD_REQUEST, "invalid type");
    };
    let (period, dir, _) = period_of(&q);
    let path = st.cfg.wordlist_dir.join(dir).join(file);
    let (path, mime, name) = if is_rule(wtype) {
        (path, "text/plain", format!("honey_{period}_{file}"))
    } else {
        (
            gz_path(&path),
            "application/gzip",
            format!("honey_{period}_{wtype}.txt.gz"),
        )
    };

    let body = match tokio::fs::read(&path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return error(
                StatusCode::SERVICE_UNAVAILABLE,
                "wordlist not yet generated",
            );
        }
        body => body,
    };

    match body {
        Ok(body) => (
            [
                (header::CONTENT_TYPE, mime.to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename={name}"),
                ),
            ],
            body,
        )
            .into_response(),
        Err(e) => {
            eprintln!("download {wtype}/{period}: {e}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "cannot read wordlist")
        }
    }
}
