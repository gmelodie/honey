use std::path::PathBuf;
use std::sync::Arc;

use honey::env_or;
use honey::live::Live;
use honey::web::{Config, router};
use tokio::signal::unix::{SignalKind, signal};

async fn shutdown(live: Arc<Live>) {
    let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    tokio::select! {
        _ = term.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    live.close();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config {
        stats_dir: PathBuf::from(env_or("STATS_DIR", "/stats")),
        wordlist_dir: PathBuf::from(env_or("WORDLIST_DIR", "/wordlists")),
        recaptcha_site_key: env_or("RECAPTCHA_SITE_KEY", ""),
        recaptcha_secret: env_or("RECAPTCHA_SECRET_KEY", ""),
        recaptcha_min_score: env_or("RECAPTCHA_MIN_SCORE", "0.5").parse()?,
        recaptcha_verify_url: "https://www.google.com/recaptcha/api/siteverify".into(),
    };
    let bind = env_or("BIND", "127.0.0.1:8373");
    let live = Live::spawn(env_or("COWRIE_LOG", "/cowrie-log/cowrie.json").into());

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    println!("listening on {bind}");
    axum::serve(listener, router(cfg, live.clone()))
        .with_graceful_shutdown(shutdown(live))
        .await?;
    Ok(())
}
