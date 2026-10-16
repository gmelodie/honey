use std::io::Read;
use std::path::Path;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use honey::web::{Config, fmt_bytes, render_index, router};
use serde_json::{Value, json};
use tower::ServiceExt;

struct Env {
    _dir: tempfile::TempDir,
    stats: std::path::PathBuf,
    words: std::path::PathBuf,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let stats = dir.path().join("stats");
    let words = dir.path().join("words");
    std::fs::create_dir_all(&stats).unwrap();
    std::fs::create_dir_all(&words).unwrap();
    Env {
        _dir: dir,
        stats,
        words,
    }
}

fn app(e: &Env, secret: &str, url: &str) -> Router {
    router(Config {
        stats_dir: e.stats.clone(),
        wordlist_dir: e.words.clone(),
        recaptcha_site_key: "site".into(),
        recaptcha_secret: secret.into(),
        recaptcha_min_score: 0.5,
        recaptcha_verify_url: url.into(),
    })
}

async fn send(
    app: Router,
    method: Method,
    uri: &str,
    body: &str,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let (parts, body) = resp.into_parts();
    (
        parts.status,
        parts.headers,
        to_bytes(body, usize::MAX).await.unwrap().to_vec(),
    )
}

async fn get(app: Router, uri: &str) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    send(app, Method::GET, uri, "").await
}

fn put(root: &Path, dir: &str, file: &str, data: &str) {
    let path = root.join(dir).join(file);
    std::fs::create_dir_all(root.join(dir)).unwrap();
    std::fs::write(&path, data).unwrap();
    if path.extension().is_some_and(|x| x == "txt") {
        std::fs::write(honey::gz_path(&path), honey::gzip(data.as_bytes()).unwrap()).unwrap();
    }
}

fn js(b: &[u8]) -> Value {
    serde_json::from_slice(b).unwrap()
}

#[test]
fn fmt_bytes_table() {
    let cases = [
        (0, "0 B"),
        (1023, "1023 B"),
        (1024, "1 KB"),
        (1536, "2 KB"),
        (2560, "2 KB"),
        (1024 * 1024 - 1, "1024 KB"),
        (1024 * 1024, "1 MB"),
        (1024 * 1024 * 1024, "1.0 GB"),
        (5 * 1024 * 1024 * 1024 + 512 * 1024 * 1024, "5.5 GB"),
        (1u64 << 50, "1048576.0 GB"),
    ];
    for (n, want) in cases {
        assert_eq!(fmt_bytes(n), want, "{n}");
    }
}

#[test]
fn render_index_with_empty_key_uses_explicit_render_and_empty_json() {
    let html = render_index("");
    assert!(html.contains("render=explicit"));
    assert!(html.contains("window.__RC_SITE_KEY__ = \"\";"));
    assert!(!html.contains("{{"));
}

#[test]
fn render_index_escapes_key_in_attribute_and_script_contexts() {
    let html = render_index("a&b<\"'>");
    assert!(html.contains("render=a&amp;b&lt;&quot;&#39;&gt;"));
    assert!(html.contains(r#"window.__RC_SITE_KEY__ = "a\u0026b\u003c\"\u0027\u003e";"#));
}

#[tokio::test]
async fn index_serves_rendered_html() {
    let e = env();
    let (st, h, body) = get(app(&e, "", ""), "/").await;
    assert_eq!(st, StatusCode::OK);
    assert!(
        h[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    assert_eq!(String::from_utf8(body).unwrap(), render_index("site"));
}

#[tokio::test]
async fn stats_serves_requested_window_file_verbatim() {
    let e = env();
    std::fs::write(e.stats.join("24h.json"), r#"{"w":"24h"}"#).unwrap();
    let (st, h, body) = get(app(&e, "", ""), "/api/stats?window=24h").await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h[header::CONTENT_TYPE], "application/json");
    assert_eq!(body, br#"{"w":"24h"}"#);
}

#[tokio::test]
async fn stats_bad_or_missing_window_falls_back_to_6h() {
    let e = env();
    std::fs::write(e.stats.join("6h.json"), r#"{"w":"6h"}"#).unwrap();
    std::fs::write(e.stats.join("secret.json"), "leak").unwrap();
    for q in [
        "",
        "?window=bogus",
        "?window=",
        "?window=../stats/secret",
        "?window=6H",
        "?window=secret",
    ] {
        let (st, _, body) = get(app(&e, "", ""), &format!("/api/stats{q}")).await;
        assert_eq!(
            (st, body.as_slice()),
            (StatusCode::OK, br#"{"w":"6h"}"#.as_slice()),
            "{q}"
        );
    }
}

#[tokio::test]
async fn stats_missing_file_is_503_json() {
    let e = env();
    let (st, _, body) = get(app(&e, "", ""), "/api/stats?window=7d").await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        js(&body),
        json!({"error": "stats not yet generated, please wait"})
    );
}

#[tokio::test]
async fn stats_unreadable_file_is_500() {
    let e = env();
    std::fs::create_dir(e.stats.join("1h.json")).unwrap();
    let (st, _, body) = get(app(&e, "", ""), "/api/stats?window=1h").await;
    assert_eq!(st, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(js(&body), json!({"error": "cannot read stats"}));
}

#[tokio::test]
async fn wordlist_meta_all_missing_reports_not_ready() {
    let e = env();
    let (st, _, body) = get(app(&e, "", ""), "/api/wordlist").await;
    let v = js(&body);
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["period"], "all");
    for k in [
        "usernames",
        "passwords",
        "pairs",
        "novel_passwords",
        "trending_passwords",
        "dying_passwords",
        "hashcat_rules",
        "john_rules",
    ] {
        assert_eq!(
            v[k],
            json!({"ready": false, "oldest": null, "newest": null}),
            "{k}"
        );
    }
}

#[tokio::test]
async fn wordlist_meta_plain_list_counts_lines_previews_five_and_sets_dates() {
    let e = env();
    put(&e.words, "daily", "passwords.txt", "a\nb\nc\nd\ne\nf\ng");
    put(&e.words, "daily", "usernames.txt", "");
    let (_, _, body) = get(app(&e, "", ""), "/api/wordlist?period=daily").await;
    let v = js(&body);
    assert_eq!(v["period"], "daily");
    let p = &v["passwords"];
    assert_eq!(p["ready"], true);
    assert_eq!(p["total"], 7);
    assert_eq!(p["preview"], json!(["a", "b", "c", "d", "e"]));
    assert_eq!(p["size"], "13 B");
    let gz_len = honey::gzip(b"a\nb\nc\nd\ne\nf\ng").unwrap().len() as u64;
    assert_eq!(p["gz_size"], fmt_bytes(gz_len));
    assert!(p["oldest"].as_str().unwrap().ends_with("+00:00"));
    assert!(p["oldest"].as_str().unwrap() < p["newest"].as_str().unwrap());
    assert_eq!(v["usernames"]["total"], 0);
    assert_eq!(v["usernames"]["preview"], json!([]));
}

#[tokio::test]
async fn wordlist_meta_all_period_has_null_oldest_and_unknown_period_means_all() {
    let e = env();
    put(&e.words, "alltime", "pairs.txt", "x");
    put(&e.words, "alltime", "passwords_usernames.txt", "p:u\n");
    for q in [
        "",
        "?period=all",
        "?period=yearly",
        "?period=alltime",
        "?period=",
    ] {
        let (_, _, body) = get(app(&e, "", ""), &format!("/api/wordlist{q}")).await;
        let v = js(&body);
        assert_eq!(v["period"], "all", "{q}");
        assert_eq!(v["pairs"]["ready"], true, "{q}");
        assert_eq!(v["pairs"]["oldest"], Value::Null, "{q}");
        assert!(v["pairs"]["newest"].is_string(), "{q}");
    }
}

#[tokio::test]
async fn wordlist_meta_rule_files_skip_comments_and_have_null_dates() {
    let e = env();
    put(
        &e.words,
        "weekly",
        "hashcat.rule",
        "# c\n\nl\n# x\n$1\n[sect]\nc $1\n$2\n$3\n$4\n$5\n",
    );
    put(
        &e.words,
        "weekly",
        "john.rule",
        "[List.Rules:autopot]\n# c\nAz\"1\"\n",
    );
    let (_, _, body) = get(app(&e, "", ""), "/api/wordlist?period=weekly").await;
    let v = js(&body);
    assert_eq!(
        v["hashcat_rules"]["preview"],
        json!(["l", "$1", "c $1", "$2", "$3"])
    );
    assert_eq!(v["hashcat_rules"]["total"], 11);
    assert_eq!(v["hashcat_rules"]["oldest"], Value::Null);
    assert_eq!(v["hashcat_rules"]["newest"], Value::Null);
    assert_eq!(v["john_rules"]["preview"], json!(["Az\"1\""]));
}

#[tokio::test]
async fn wordlist_meta_non_utf8_bytes_are_replaced_not_fatal() {
    let e = env();
    std::fs::create_dir_all(e.words.join("alltime")).unwrap();
    std::fs::write(e.words.join("alltime/passwords.txt"), b"ok\n\xff\xfe\n").unwrap();
    let (st, _, body) = get(app(&e, "", ""), "/api/wordlist").await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        js(&body)["passwords"]["preview"],
        json!(["ok", "\u{fffd}\u{fffd}"])
    );
}

#[tokio::test]
async fn wordlist_meta_directory_in_place_of_file_is_not_ready() {
    let e = env();
    std::fs::create_dir_all(e.words.join("alltime/passwords.txt")).unwrap();
    let (_, _, body) = get(app(&e, "", ""), "/api/wordlist").await;
    assert_eq!(js(&body)["passwords"]["ready"], false);
}

#[tokio::test]
async fn download_plain_list_is_gzip_that_round_trips() {
    let e = env();
    let data = "pw1\npw2\n\u{e9}\n";
    put(&e.words, "monthly", "passwords.txt", data);
    let (st, h, body) = get(
        app(&e, "", ""),
        "/api/wordlist/passwords/download?period=monthly",
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h[header::CONTENT_TYPE], "application/gzip");
    assert_eq!(
        h[header::CONTENT_DISPOSITION],
        "attachment; filename=honey_monthly_passwords.txt.gz"
    );
    let mut out = String::new();
    flate2::read::GzDecoder::new(body.as_slice())
        .read_to_string(&mut out)
        .unwrap();
    assert_eq!(out, data);
}

#[tokio::test]
async fn download_pairs_uses_url_type_in_name_and_pairs_file() {
    let e = env();
    put(&e.words, "alltime", "passwords_usernames.txt", "p:u\n");
    let (st, h, _) = get(app(&e, "", ""), "/api/wordlist/pairs/download").await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        h[header::CONTENT_DISPOSITION],
        "attachment; filename=honey_all_pairs.txt.gz"
    );
}

#[tokio::test]
async fn download_rule_file_is_raw_text_with_rule_filename() {
    let e = env();
    put(&e.words, "daily", "hashcat.rule", "# h\n$1\n");
    put(&e.words, "daily", "john.rule", "[x]\n");
    let (st, h, body) = get(
        app(&e, "", ""),
        "/api/wordlist/hashcat_rules/download?period=daily",
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h[header::CONTENT_TYPE], "text/plain");
    assert_eq!(
        h[header::CONTENT_DISPOSITION],
        "attachment; filename=honey_daily_hashcat.rule"
    );
    assert_eq!(body, b"# h\n$1\n");
    let (_, h, body) = get(
        app(&e, "", ""),
        "/api/wordlist/john_rules/download?period=daily",
    )
    .await;
    assert_eq!(
        h[header::CONTENT_DISPOSITION],
        "attachment; filename=honey_daily_john.rule"
    );
    assert_eq!(body, b"[x]\n");
}

#[tokio::test]
async fn download_empty_list_gzips_to_empty_content() {
    let e = env();
    put(&e.words, "alltime", "usernames.txt", "");
    let (st, _, body) = get(app(&e, "", ""), "/api/wordlist/usernames/download").await;
    assert_eq!(st, StatusCode::OK);
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(body.as_slice())
        .read_to_end(&mut out)
        .unwrap();
    assert!(out.is_empty());
}

#[tokio::test]
async fn download_invalid_type_is_400_and_checked_before_file_lookup() {
    let e = env();
    for t in ["bogus", "passwords.txt", "..", "Passwords"] {
        let (st, _, body) = get(app(&e, "", ""), &format!("/api/wordlist/{t}/download")).await;
        assert_eq!(
            (st, js(&body)),
            (StatusCode::BAD_REQUEST, json!({"error": "invalid type"})),
            "{t}"
        );
    }
}

#[tokio::test]
async fn download_missing_file_is_503() {
    let e = env();
    let (st, _, body) = get(
        app(&e, "", ""),
        "/api/wordlist/passwords/download?period=weekly",
    )
    .await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(js(&body), json!({"error": "wordlist not yet generated"}));
}

#[tokio::test]
async fn download_unknown_period_falls_back_to_all() {
    let e = env();
    put(&e.words, "alltime", "hashcat.rule", "r\n");
    let (st, h, body) = get(
        app(&e, "", ""),
        "/api/wordlist/hashcat_rules/download?period=../daily",
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        h[header::CONTENT_DISPOSITION],
        "attachment; filename=honey_all_hashcat.rule"
    );
    assert_eq!(body, b"r\n");
}

#[tokio::test]
async fn download_directory_in_place_of_file_is_500() {
    let e = env();
    std::fs::create_dir_all(e.words.join("alltime/hashcat.rule")).unwrap();
    let (st, _, body) = get(app(&e, "", ""), "/api/wordlist/hashcat_rules/download").await;
    assert_eq!(st, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(js(&body), json!({"error": "cannot read wordlist"}));
}

#[tokio::test]
async fn list_without_precomputed_gzip_is_503_and_has_null_gz_size() {
    let e = env();
    std::fs::create_dir_all(e.words.join("alltime")).unwrap();
    std::fs::write(e.words.join("alltime/passwords.txt"), "a\n").unwrap();
    let (st, _, _) = get(app(&e, "", ""), "/api/wordlist/passwords/download").await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    let (_, _, body) = get(app(&e, "", ""), "/api/wordlist").await;
    assert_eq!(js(&body)["passwords"]["gz_size"], Value::Null);
}

#[tokio::test]
async fn unknown_api_paths_are_json_404() {
    let e = env();
    for p in [
        "/api/nope",
        "/api/",
        "/api/wordlist/passwords",
        "/api/wordlist/passwords/download/x",
    ] {
        let (st, _, body) = get(app(&e, "", ""), p).await;
        assert_eq!(
            (st, js(&body)),
            (StatusCode::NOT_FOUND, json!({"error": "not found"})),
            "{p}"
        );
    }
}

#[tokio::test]
async fn other_paths_redirect_to_root() {
    let e = env();
    for p in ["/nope", "/apix", "/api", "/a/b/c"] {
        let (st, h, _) = get(app(&e, "", ""), p).await;
        assert_eq!(
            (st, h[header::LOCATION].to_str().unwrap()),
            (StatusCode::FOUND, "/"),
            "{p}"
        );
    }
}

#[tokio::test]
async fn wrong_method_on_known_route_is_405() {
    let e = env();
    let (st, _, _) = send(app(&e, "", ""), Method::POST, "/api/stats", "").await;
    assert_eq!(st, StatusCode::METHOD_NOT_ALLOWED);
    let (st, _, _) = get(app(&e, "", ""), "/api/verify-captcha").await;
    assert_eq!(st, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn static_files_have_matching_content_types_and_bodies() {
    let e = env();
    let cases = [
        (
            "/static/css/style.css",
            "text/css; charset=utf-8",
            "web/static/css/style.css",
        ),
        (
            "/static/js/app.js",
            "text/javascript; charset=utf-8",
            "web/static/js/app.js",
        ),
        (
            "/static/favicon.svg",
            "image/svg+xml",
            "web/static/favicon.svg",
        ),
    ];
    for (url, mime, file) in cases {
        let (st, h, body) = get(app(&e, "", ""), url).await;
        assert_eq!(st, StatusCode::OK, "{url}");
        assert_eq!(h[header::CONTENT_TYPE], mime, "{url}");
        assert_eq!(
            body,
            std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).unwrap(),
            "{url}"
        );
    }
}

#[tokio::test]
async fn static_unknown_or_traversal_is_json_404() {
    let e = env();
    for p in [
        "/static/nope.js",
        "/static/css/../../Cargo.toml",
        "/static/%2e%2e/Cargo.toml",
        "/static/css/",
    ] {
        let (st, _, body) = get(app(&e, "", ""), p).await;
        assert_eq!(
            (st, js(&body)),
            (StatusCode::NOT_FOUND, json!({"error": "not found"})),
            "{p}"
        );
    }
}

#[tokio::test]
async fn verify_captcha_with_empty_secret_succeeds_without_body() {
    let e = env();
    let (st, _, body) = send(
        app(&e, "", "http://127.0.0.1:1/"),
        Method::POST,
        "/api/verify-captcha",
        "",
    )
    .await;
    assert_eq!((st, js(&body)), (StatusCode::OK, json!({"success": true})));
}

#[tokio::test]
async fn verify_captcha_with_secret_rejects_missing_empty_or_bad_token_without_network() {
    let e = env();
    for b in [
        "",
        "not json",
        "{}",
        r#"{"token": ""}"#,
        r#"{"token": 5}"#,
        "[]",
    ] {
        let (st, _, body) = send(
            app(&e, "s", "http://127.0.0.1:1/"),
            Method::POST,
            "/api/verify-captcha",
            b,
        )
        .await;
        assert_eq!(
            (st, js(&body)),
            (StatusCode::BAD_REQUEST, json!({"success": false})),
            "{b:?}"
        );
    }
}

#[tokio::test]
async fn verify_captcha_unreachable_verifier_is_502() {
    let e = env();
    let (st, _, body) = send(
        app(&e, "s", "http://127.0.0.1:1/"),
        Method::POST,
        "/api/verify-captcha",
        r#"{"token":"t"}"#,
    )
    .await;
    assert_eq!(
        (st, js(&body)),
        (StatusCode::BAD_GATEWAY, json!({"success": false}))
    );
}

async fn fake_verifier(
    status: u16,
    reply: &'static str,
) -> (String, tokio::task::JoinHandle<String>) {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", l.local_addr().unwrap());
    let h = tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let (mut s, _) = l.accept().await.unwrap();
        let mut buf = vec![0u8; 8192];
        let mut got = Vec::new();
        loop {
            let n = s.read(&mut buf).await.unwrap();
            got.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&got).to_string();
            if let Some((head, rest)) = text.split_once("\r\n\r\n") {
                let len = head
                    .to_lowercase()
                    .split("content-length: ")
                    .nth(1)
                    .and_then(|r| r.lines().next()?.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                if rest.len() >= len || n == 0 {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        let resp = format!(
            "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
            reply.len()
        );
        s.write_all(resp.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&got).to_string()
    });
    (url, h)
}

#[tokio::test]
async fn verify_captcha_score_threshold_and_form_body() {
    let e = env();
    let cases = [
        (r#"{"success":true,"score":0.5}"#, true),
        (r#"{"success":true,"score":0.49}"#, false),
        (r#"{"success":true}"#, false),
        (r#"{"success":false,"score":0.9}"#, false),
        ("garbage", false),
    ];
    for (reply, want) in cases {
        let (url, h) = fake_verifier(200, reply).await;
        let (st, _, body) = send(
            app(&e, "sec&ret", &url),
            Method::POST,
            "/api/verify-captcha",
            r#"{"token":"t k"}"#,
        )
        .await;
        assert_eq!(
            (st, js(&body)),
            (StatusCode::OK, json!({"success": want})),
            "{reply}"
        );
        let req = h.await.unwrap();
        assert!(req.starts_with("POST / "), "{req}");
        assert!(req.ends_with("secret=sec%26ret&response=t+k"), "{req}");
    }
}

#[tokio::test]
async fn verify_captcha_verifier_http_error_is_502() {
    let e = env();
    let (url, _h) = fake_verifier(500, "{}").await;
    let (st, _, body) = send(
        app(&e, "s", &url),
        Method::POST,
        "/api/verify-captcha",
        r#"{"token":"t"}"#,
    )
    .await;
    assert_eq!(
        (st, js(&body)),
        (StatusCode::BAD_GATEWAY, json!({"success": false}))
    );
}
