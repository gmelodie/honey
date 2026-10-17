//! Imports Cowrie JSON log files into `PostgreSQL`.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::ExitCode;

use anyhow::{Context, Result};
use honey::cowrie::Batch;
use honey::db::{self, parse_dotenv};
use postgres::Transaction;
use serde_json::Value;

const DEFAULTS: [(&str, &str); 4] = [
    ("POSTGRES_HOST", "127.0.0.1"),
    ("POSTGRES_PORT", "5432"),
    ("POSTGRES_DB", "cowrie"),
    ("POSTGRES_USER", "cowrie"),
];

fn read_events(path: &str) -> Result<Vec<Value>> {
    let mut events = Vec::new();
    for (lineno, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str(line) {
            Ok(ev) => events.push(ev),
            Err(e) => eprintln!("  skip {path}:{}: {e}", lineno + 1),
        }
    }
    Ok(events)
}

fn id_map(tx: &mut Transaction, sql: &str) -> Result<HashMap<String, i32>> {
    Ok(tx
        .query(sql, &[])?
        .iter()
        .filter_map(|r| Some((r.get::<_, Option<String>>(0)?, r.get(1))))
        .collect())
}

fn col<T, U>(rows: &[T], f: impl Fn(&T) -> U) -> Vec<U> {
    rows.iter().map(f).collect()
}

#[expect(
    clippy::many_single_char_names,
    reason = "names match the SQL table aliases"
)]
fn insert(tx: &mut Transaction, b: &Batch) -> Result<()> {
    tx.execute(
        "INSERT INTO sensors (ip) SELECT unnest($1::text[]) ON CONFLICT (ip) DO NOTHING",
        &[&b.sensors],
    )?;
    let sensor_id = id_map(tx, "SELECT ip, id FROM sensors")?;

    let versions: Vec<&String> = b.clients.iter().filter(|v| !v.is_empty()).collect();
    tx.execute(
        "INSERT INTO clients (version)
         SELECT v FROM unnest($1::text[]) AS v
         WHERE NOT EXISTS (SELECT 1 FROM clients c WHERE c.version = v)",
        &[&versions],
    )?;
    let client_id = id_map(tx, "SELECT version, min(id) FROM clients GROUP BY version")?;

    let s = &b.sessions;
    tx.execute(
        "INSERT INTO sessions (id, starttime, endtime, sensor, ip, termsize, client, protocol)
         SELECT id, st::timestamptz, et::timestamptz, sensor, ip, termsize, client, proto
         FROM unnest($1::text[], $2::text[], $3::text[], $4::int4[], $5::text[], $6::text[],
                     $7::int4[], $8::text[])
              AS t(id, st, et, sensor, ip, termsize, client, proto)
         ON CONFLICT (id) DO NOTHING",
        &[
            &col(s, |x| x.id.clone()),
            &col(s, |x| x.starttime.clone()),
            &col(s, |x| x.endtime.clone()),
            &col(s, |x| sensor_id.get(&x.sensor).copied()),
            &col(s, |x| x.ip.clone()),
            &col(s, |x| x.termsize.clone()),
            &col(s, |x| {
                x.client.as_ref().and_then(|c| client_id.get(c).copied())
            }),
            &col(s, |x| x.protocol.clone()),
        ],
    )?;

    let a = &b.auth;
    tx.execute(
        "INSERT INTO auth (session, success, username, password, timestamp)
         SELECT s, ok, u, p, ts::timestamptz
         FROM unnest($1::text[], $2::bool[], $3::text[], $4::text[], $5::text[]) AS t(s, ok, u, p, ts)",
        &[
            &col(a, |r| r.session.clone()),
            &col(a, |r| r.success),
            &col(a, |r| r.username.clone()),
            &col(a, |r| r.password.clone()),
            &col(a, |r| r.timestamp.clone()),
        ],
    )?;

    let i = &b.input;
    tx.execute(
        "INSERT INTO input (session, timestamp, realm, input, success)
         SELECT s, ts::timestamptz, realm, input, ok
         FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::bool[]) AS t(s, ts, realm, input, ok)",
        &[
            &col(i, |r| r.session.clone()),
            &col(i, |r| r.timestamp.clone()),
            &col(i, |r| r.realm.clone()),
            &col(i, |r| r.input.clone()),
            &col(i, |r| r.success),
        ],
    )?;

    tx.execute(
        "INSERT INTO ttylog (session, ttylog) SELECT * FROM unnest($1::text[], $2::text[])",
        &[
            &col(&b.ttylogs, |t| t.0.clone()),
            &col(&b.ttylogs, |t| t.1.clone()),
        ],
    )?;

    let d = &b.downloads;
    tx.execute(
        "INSERT INTO downloads (session, timestamp, url, outfile, shasum, source)
         SELECT s, ts::timestamptz, url, outfile, sha, src
         FROM unnest($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[])
              AS t(s, ts, url, outfile, sha, src)",
        &[
            &col(d, |r| r.session.clone()),
            &col(d, |r| r.timestamp.clone()),
            &col(d, |r| r.url.clone()),
            &col(d, |r| r.outfile.clone()),
            &col(d, |r| r.shasum.clone()),
            &col(d, |r| r.source.clone()),
        ],
    )?;

    let f = &b.fingerprints;
    tx.execute(
        "INSERT INTO keyfingerprints (session, username, fingerprint)
         SELECT * FROM unnest($1::text[], $2::text[], $3::text[])",
        &[
            &col(f, |r| r.session.clone()),
            &col(f, |r| r.username.clone()),
            &col(f, |r| r.fingerprint.clone()),
        ],
    )?;
    Ok(())
}

const TABLES: [&str; 8] = [
    "sensors",
    "clients",
    "sessions",
    "auth",
    "input",
    "ttylog",
    "downloads",
    "fingerprints",
];

fn counts(b: &Batch) -> [usize; 8] {
    [
        b.sensors.len(),
        b.clients.len(),
        b.sessions.len(),
        b.auth.len(),
        b.input.len(),
        b.ttylogs.len(),
        b.downloads.len(),
        b.fingerprints.len(),
    ]
}

fn run(paths: &[String]) -> Result<()> {
    if let Some(missing) = paths.iter().find(|p| !std::path::Path::new(p).exists()) {
        anyhow::bail!("file not found: {missing}");
    }

    let dotenv = std::fs::read_to_string(".env")
        .map(|t| parse_dotenv(&t))
        .unwrap_or_default();
    let mut db = db::connect(&dotenv, &DEFAULTS).context("connect")?;

    let mut totals = [0usize; 8];
    for path in paths {
        println!("Processing {path}...");
        let batch = Batch::collect(read_events(path).with_context(|| format!("read {path}"))?);
        let mut tx = db.transaction()?;
        insert(&mut tx, &batch).with_context(|| format!("import {path}"))?;
        tx.commit()?;

        for (total, n) in totals.iter_mut().zip(counts(&batch)) {
            *total += n;
        }
        println!(
            "  sessions={} auth={} input={}",
            batch.sessions.len(),
            batch.auth.len(),
            batch.input.len()
        );
    }

    println!("\nTotal:");
    for (name, n) in TABLES.iter().zip(totals) {
        println!("  {name:<14} {n}");
    }
    Ok(())
}

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: honey-import <cowrie.json> [more files...]");
        return ExitCode::from(2);
    }
    match run(&paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
