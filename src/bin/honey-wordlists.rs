//! Writes daily, weekly, monthly and all-time credential wordlists and mutation rules.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use honey::bloom::ScalableBloom;
use honey::db::{self, HOST_PORT_DEFAULTS};
use honey::rules::generate_rules;
use honey::trends::{self, TrendRow};
use honey::{env_or, gz_path, gzip, ts, write_atomic};
use postgres::Client;

struct Window {
    name: &'static str,
    since: Option<&'static str>,
    baseline: Option<&'static str>,
}

const WINDOWS: [Window; 4] = [
    Window {
        name: "daily",
        since: Some("NOW() - INTERVAL '1 day'"),
        baseline: Some("NOW() - INTERVAL '2 days'"),
    },
    Window {
        name: "weekly",
        since: Some("NOW() - INTERVAL '7 days'"),
        baseline: Some("NOW() - INTERVAL '14 days'"),
    },
    Window {
        name: "monthly",
        since: Some("NOW() - INTERVAL '30 days'"),
        baseline: Some("NOW() - INTERVAL '60 days'"),
    },
    Window {
        name: "alltime",
        since: None,
        baseline: None,
    },
];

fn since_clause(since: Option<&str>) -> String {
    since_col_clause("\"timestamp\"", since)
}

fn since_col_clause(col: &str, since: Option<&str>) -> String {
    since
        .map(|s| format!("AND {col} >= {s}"))
        .unwrap_or_default()
}

fn fetch_ranked(db: &mut Client, column: &str, since: Option<&str>) -> Result<Vec<String>> {
    let sql = format!(
        "SELECT {column} FROM auth
         WHERE {column} IS NOT NULL AND {column} <> '' {}
         GROUP BY {column} ORDER BY COUNT(*) DESC, {column}",
        since_clause(since)
    );
    Ok(db.query(&sql, &[])?.iter().map(|r| r.get(0)).collect())
}

fn fetch_pairs(db: &mut Client, since: Option<&str>) -> Result<Vec<String>> {
    let sql = format!(
        "SELECT password || ':' || username AS pair FROM auth
         WHERE username IS NOT NULL AND username <> ''
           AND password IS NOT NULL AND password <> '' {}
         GROUP BY pair ORDER BY COUNT(*) DESC, pair",
        since_clause(since)
    );
    Ok(db.query(&sql, &[])?.iter().map(|r| r.get(0)).collect())
}

fn fetch_trend_rows(db: &mut Client, recent: &str, baseline: &str) -> Result<Vec<TrendRow>> {
    let sql = format!(
        "SELECT password,
                COUNT(*) FILTER (WHERE \"timestamp\" >= {recent}),
                COUNT(*) FILTER (WHERE \"timestamp\" >= {baseline} AND \"timestamp\" < {recent})
         FROM auth
         WHERE password IS NOT NULL AND password <> '' AND \"timestamp\" >= {baseline}
         GROUP BY password"
    );
    Ok(db
        .query(&sql, &[])?
        .iter()
        .map(|r| TrendRow {
            password: r.get(0),
            recent: r.get(1),
            baseline: r.get(2),
        })
        .collect())
}

fn fetch_ips(db: &mut Client, since: Option<&str>) -> Result<Vec<String>> {
    let sql = format!(
        "SELECT ip FROM sessions
         WHERE ip IS NOT NULL AND ip <> '' {}
         GROUP BY ip ORDER BY COUNT(*) DESC, ip",
        since_col_clause("starttime", since)
    );
    Ok(db.query(&sql, &[])?.iter().map(|r| r.get(0)).collect())
}

fn write_lines<S: AsRef<str>>(path: &Path, lines: &[S]) -> Result<()> {
    let mut text = String::new();
    for l in lines {
        text.push_str(l.as_ref());
        text.push('\n');
    }
    write_atomic(path, text.as_bytes())?;

    write_atomic(&gz_path(path), &gzip(text.as_bytes())?)?;
    Ok(())
}

fn load_bloom(path: &Path) -> Option<ScalableBloom> {
    let f = File::open(path).ok()?;
    match ScalableBloom::read_from(&mut BufReader::new(f)) {
        Ok(bf) => Some(bf),
        Err(e) => {
            eprintln!(
                "[{}] WARNING: cannot read Bloom filter {}: {e}",
                ts(),
                path.display()
            );
            None
        }
    }
}

fn generate(
    db: &mut Client,
    w: &Window,
    dir: &Path,
    bloom: Option<&ScalableBloom>,
) -> Result<String> {
    let passwords = fetch_ranked(db, "password", w.since)?;
    let usernames = fetch_ranked(db, "username", w.since)?;
    let pairs = fetch_pairs(db, w.since)?;
    let ips = fetch_ips(db, w.since)?;
    write_lines(&dir.join("passwords.txt"), &passwords)?;
    write_lines(&dir.join("usernames.txt"), &usernames)?;
    write_lines(&dir.join("passwords_usernames.txt"), &pairs)?;
    write_lines(&dir.join("ips.txt"), &ips)?;

    let (hashcat, john) = generate_rules(&passwords);
    write_atomic(&dir.join("hashcat.rule"), hashcat.as_bytes())?;
    write_atomic(&dir.join("john.rule"), john.as_bytes())?;

    let mut summary = format!(
        "{} passwords, {} usernames, {} pairs, {} ips",
        passwords.len(),
        usernames.len(),
        pairs.len(),
        ips.len()
    );

    if let Some(bf) = bloom {
        let novel: Vec<&String> = passwords.iter().filter(|p| !bf.contains(p)).collect();
        write_lines(&dir.join("novel_passwords.txt"), &novel)?;
        write!(summary, ", {} novel", novel.len())?;
    }

    let t = match (w.since, w.baseline) {
        (Some(recent), Some(baseline)) => {
            let t = trends::classify(fetch_trend_rows(db, recent, baseline)?);
            write!(
                summary,
                ", {} trending, {} dying (floor={})",
                t.trending.len(),
                t.dying.len(),
                t.min_count
            )?;
            t
        }
        _ => trends::classify(Vec::new()),
    };
    write_lines(&dir.join("trending_passwords.txt"), &t.trending)?;
    write_lines(&dir.join("dying_passwords.txt"), &t.dying)?;

    Ok(summary)
}

fn run() -> Result<()> {
    let root = PathBuf::from(env_or("WORDLIST_DIR", "/wordlists"));
    let bloom_path = PathBuf::from(env_or("BLOOM_DIR", "/bloom")).join("reference.bloom");

    let bloom = load_bloom(&bloom_path);
    if bloom.is_none() {
        println!(
            "[{}] No Bloom filter at {}; novel_passwords.txt will be skipped.",
            ts(),
            bloom_path.display()
        );
    }

    let mut db = db::connect(&HashMap::default(), &HOST_PORT_DEFAULTS)?;
    for w in &WINDOWS {
        let summary = generate(&mut db, w, &root.join(w.name), bloom.as_ref())?;
        println!("[{}] {}: {summary}", ts(), w.name);
    }
    Ok(())
}

fn main() -> ExitCode {
    println!("[{}] Starting wordlist generation...", ts());
    match run() {
        Ok(()) => {
            println!("[{}] Done.", ts());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[{}] ERROR: {e:#}", ts());
            ExitCode::FAILURE
        }
    }
}
