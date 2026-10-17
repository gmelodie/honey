//! Precomputes `/api/stats` for each time window into `STATS_DIR/{window}.json`.

mod campaign;
mod geoip;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use honey::db::{self, HOST_PORT_DEFAULTS, rows_json};
use honey::{env_or, iso, ts, write_atomic};
use postgres::Client;
use serde_json::{Value, json};

struct Window {
    name: &'static str,
    span: Option<Duration>,
    bucket: &'static str,
    wordlist_period: &'static str,
}

const WINDOWS: [Window; 6] = [
    Window {
        name: "1h",
        span: Some(Duration::hours(1)),
        bucket: "date_trunc('hour', a.timestamp) + (EXTRACT(MINUTE FROM a.timestamp)::int / 5) * INTERVAL '5 minutes'",
        wordlist_period: "daily",
    },
    Window {
        name: "6h",
        span: Some(Duration::hours(6)),
        bucket: "date_trunc('hour', a.timestamp) + (EXTRACT(MINUTE FROM a.timestamp)::int / 30) * INTERVAL '30 minutes'",
        wordlist_period: "daily",
    },
    Window {
        name: "24h",
        span: Some(Duration::hours(24)),
        bucket: "date_trunc('hour', a.timestamp)",
        wordlist_period: "daily",
    },
    Window {
        name: "7d",
        span: Some(Duration::days(7)),
        bucket: "date_trunc('day', a.timestamp) + (EXTRACT(HOUR FROM a.timestamp)::int / 6) * INTERVAL '6 hours'",
        wordlist_period: "weekly",
    },
    Window {
        name: "30d",
        span: Some(Duration::days(30)),
        bucket: "date_trunc('day', a.timestamp)",
        wordlist_period: "monthly",
    },
    Window {
        name: "all",
        span: None,
        bucket: "date_trunc('day', a.timestamp)",
        wordlist_period: "alltime",
    },
];

const CAMPAIGN_WINDOWS: [&str; 3] = ["6h", "24h", "7d"];

/// A NULL `$1` means all time.
fn since(col: &str) -> String {
    format!("($1::timestamptz IS NULL OR {col} >= $1)")
}

struct Query<'a> {
    db: &'a mut Client,
    since: Option<DateTime<Utc>>,
}

impl Query<'_> {
    fn rows(&mut self, sql: &str) -> Result<Vec<Value>> {
        rows_json(&self.db.query(sql, &[&self.since])?)
    }

    fn count(&mut self, sql: &str) -> Result<i64> {
        Ok(self.db.query_one(sql, &[&self.since])?.get(0))
    }
}

#[expect(
    clippy::many_single_char_names,
    reason = "names match the SQL table aliases"
)]
fn compute_telnet(q: &mut Query, w: &Window) -> Result<Value> {
    let (s, a, i) = (
        since("s.starttime"),
        since("a.timestamp"),
        since("i.timestamp"),
    );
    let telnet = "s.protocol = 'telnet'";
    let auth_join = "FROM auth a JOIN sessions s ON a.session = s.id";
    let input_join = "FROM input i JOIN sessions s ON i.session = s.id";

    Ok(json!({
        "overview": {
            "connections": q.count(&format!("SELECT count(*) FROM sessions s WHERE {s} AND {telnet}"))?,
            "unique_ips": q.count(&format!("SELECT count(DISTINCT s.ip) FROM sessions s WHERE {s} AND {telnet}"))?,
            "auth_attempts": q.count(&format!("SELECT count(*) {auth_join} WHERE {a} AND {telnet}"))?,
            "unique_commands": q.count(&format!("SELECT count(DISTINCT i.input) {input_join} WHERE {i} AND {telnet}"))?,
            "cmd_sessions": q.count(&format!("SELECT count(DISTINCT i.session) {input_join} WHERE {i} AND {telnet}"))?,
            "unique_passwords": q.count(&format!("SELECT count(DISTINCT a.password) {auth_join} WHERE {a} AND {telnet}"))?,
            "unique_usernames": q.count(&format!("SELECT count(DISTINCT a.username) {auth_join} WHERE {a} AND {telnet}"))?,
        },
        "timeseries": q.rows(&format!(
            "SELECT {} AS t,
                    sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful,
                    sum(CASE WHEN NOT a.success THEN 1 ELSE 0 END) AS failed
             {auth_join} WHERE {a} AND {telnet} GROUP BY 1 ORDER BY 1", w.bucket))?,
        "top_commands": q.rows(&format!(
            "SELECT i.input AS command, count(*) AS attempts {input_join}
             WHERE {i} AND {telnet} AND i.input != ''
             GROUP BY i.input ORDER BY attempts DESC LIMIT 20"))?,
        "top_ips": q.rows(&format!(
            "SELECT s.ip, count(*) AS connections FROM sessions s WHERE {s} AND {telnet}
             GROUP BY s.ip ORDER BY connections DESC LIMIT 20"))?,
        "top_usernames": q.rows(&format!(
            "SELECT a.username, count(*) AS attempts {auth_join} WHERE {a} AND {telnet}
             GROUP BY a.username ORDER BY attempts DESC LIMIT 20"))?,
        "top_passwords": q.rows(&format!(
            "SELECT a.password, count(*) AS attempts {auth_join} WHERE {a} AND {telnet}
             GROUP BY a.password ORDER BY attempts DESC LIMIT 20"))?,
        "session_log": q.rows(&format!(
            "SELECT s.starttime AS time, s.ip, left(s.id, 8) AS session,
                    (SELECT count(*) FROM input i WHERE i.session = s.id) AS commands
             FROM sessions s WHERE {s} AND {telnet}
             ORDER BY s.starttime DESC LIMIT 100"))?,
        "auth_log": q.rows(&format!(
            "SELECT a.timestamp AS time, s.ip, a.username, a.password, a.success
             {auth_join} WHERE {a} AND {telnet} ORDER BY a.timestamp DESC LIMIT 100"))?,
    }))
}

fn round_to(x: f64, places: i32) -> f64 {
    let p = 10f64.powi(places);
    (x * p).round() / p
}

fn with_pct(mut rows: Vec<Value>, total: i64) -> Vec<Value> {
    for r in &mut rows {
        let n = r["sessions"].as_i64().unwrap_or(0);
        r["pct"] = json!(round_to(100.0 * n as f64 / total.max(1) as f64, 1));
    }
    rows
}

fn compute_country_asns(q: &mut Query, s: &str, isos: &[String]) -> Result<Value> {
    let rows = q.db.query(
        &format!(
            "SELECT country_iso, asn, asn_org, sessions FROM (
                 SELECT g.country_iso, g.asn, g.asn_org, count(*) AS sessions,
                        ROW_NUMBER() OVER (PARTITION BY g.country_iso ORDER BY count(*) DESC) AS rn
                 FROM sessions s JOIN ip_geo_cache g ON g.ip = s.ip
                 WHERE {s} AND g.asn IS NOT NULL AND g.country_iso = ANY($2)
                 GROUP BY g.country_iso, g.asn, g.asn_org
             ) ranked
             WHERE rn <= 5 ORDER BY country_iso, sessions DESC"
        ),
        &[&q.since, &isos],
    )?;

    let mut by_country = serde_json::Map::new();
    for r in rows_json(&rows)? {
        let Value::Object(mut obj) = r else { continue };
        let iso = obj
            .remove("country_iso")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        let list = by_country.entry(iso).or_insert_with(|| json!([]));
        list.as_array_mut().unwrap().push(Value::Object(obj));
    }
    Ok(Value::Object(by_country))
}

fn asn_of(row: &Value) -> Option<i32> {
    row["asn"].as_i64().and_then(|n| i32::try_from(n).ok())
}

fn compute_new_asns(q: &mut Query) -> Result<Vec<Value>> {
    if q.since.is_none() {
        return Ok(Vec::new());
    }

    let mut asns = q.rows(
        "SELECT g.asn, g.asn_org, min(s.starttime) AS first_seen, count(DISTINCT s.id) AS sessions
         FROM sessions s JOIN ip_geo_cache g ON g.ip = s.ip
         WHERE g.asn IS NOT NULL
         GROUP BY g.asn, g.asn_org
         HAVING min(s.starttime) >= $1
         ORDER BY sessions DESC LIMIT 20",
    )?;
    if asns.is_empty() {
        return Ok(asns);
    }

    let ids: Vec<i32> = asns.iter().filter_map(asn_of).collect();
    let attempts: HashMap<i32, i64> =
        q.db.query(
            "SELECT g.asn, count(*) FROM auth a
             JOIN sessions s ON a.session = s.id
             JOIN ip_geo_cache g ON g.ip = s.ip
             WHERE a.timestamp >= $1 AND g.asn = ANY($2)
             GROUP BY g.asn",
            &[&q.since, &ids],
        )?
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect();
    for r in &mut asns {
        let n = asn_of(r).and_then(|asn| attempts.get(&asn).copied());
        r["attempts"] = json!(n.unwrap_or(0));
    }
    Ok(asns)
}

fn compute_geo(q: &mut Query) -> Result<Value> {
    let s = since("s.starttime");
    let geo_join = "FROM sessions s JOIN ip_geo_cache g ON g.ip = s.ip";

    let total = q.count(&format!("SELECT count(*) FROM sessions s WHERE {s}"))?;
    let covered = q.count(&format!(
        "SELECT count(*) {geo_join} WHERE {s} AND g.country_iso IS NOT NULL"
    ))?;

    let top_countries = with_pct(
        q.rows(&format!(
            "SELECT g.country_iso, g.country_name, count(*) AS sessions {geo_join}
             WHERE {s} AND g.country_iso IS NOT NULL
             GROUP BY g.country_iso, g.country_name ORDER BY sessions DESC LIMIT 15"
        ))?,
        total,
    );
    let top_asns = with_pct(
        q.rows(&format!(
            "SELECT g.asn, g.asn_org, count(*) AS sessions {geo_join}
             WHERE {s} AND g.asn IS NOT NULL
             GROUP BY g.asn, g.asn_org ORDER BY sessions DESC LIMIT 15"
        ))?,
        total,
    );
    let top_ips = q.rows(&format!(
        "SELECT s.ip, g.country_iso, g.asn, g.asn_org,
                count(DISTINCT s.id) AS sessions, count(a.id) AS attempts
         FROM sessions s
         LEFT JOIN ip_geo_cache g ON g.ip = s.ip
         LEFT JOIN auth a ON a.session = s.id
         WHERE {s} AND s.ip IS NOT NULL
         GROUP BY s.ip, g.country_iso, g.asn, g.asn_org
         ORDER BY sessions DESC, attempts DESC LIMIT 20"
    ))?;

    let isos: Vec<String> = top_countries
        .iter()
        .filter_map(|r| r["country_iso"].as_str().map(String::from))
        .collect();
    let country_asns = if isos.is_empty() {
        json!({})
    } else {
        compute_country_asns(q, &s, &isos)?
    };

    Ok(json!({
        "top_countries": top_countries,
        "top_asns": top_asns,
        "top_ips": top_ips,
        "new_asns": compute_new_asns(q)?,
        "coverage_pct": if total > 0 { round_to(100.0 * covered as f64 / total as f64, 1) } else { 0.0 },
        "country_asns": country_asns,
    }))
}

fn count_lines(path: &Path) -> Option<usize> {
    let data = std::fs::read(path).ok()?;
    #[expect(clippy::naive_bytecount, reason = "one small file per run")]
    let newlines = data.iter().filter(|&&b| b == b'\n').count();
    Some(newlines + usize::from(data.last().is_some_and(|&b| b != b'\n')))
}

fn or_fallback(what: &str, r: Result<Value>, fallback: Value) -> Value {
    r.unwrap_or_else(|e| {
        eprintln!("WARNING: {what} stats skipped: {e:#}");
        fallback
    })
}

#[expect(
    clippy::many_single_char_names,
    reason = "names match the SQL table aliases"
)]
fn compute(db: &mut Client, w: &Window, wordlists: &Path, campaigns: &[Value]) -> Result<Value> {
    let now = Utc::now();
    let mut q = Query {
        db,
        since: w.span.map(|d| now - d),
    };
    let (a, i, s, d) = (
        since("a.timestamp"),
        since("i.timestamp"),
        since("s.starttime"),
        since("d.timestamp"),
    );

    let overview = json!({
        "connections": q.count(&format!("SELECT count(DISTINCT a.session) FROM auth a WHERE {a}"))?,
        "cmd_sessions": q.count(&format!("SELECT count(DISTINCT i.session) FROM input i WHERE {i}"))?,
        "auth_attempts": q.count(&format!("SELECT count(*) FROM auth a WHERE {a}"))?,
        "commands": q.count(&format!("SELECT count(*) FROM input i WHERE {i}"))?,
        "unique_ips": q.count(&format!("SELECT count(DISTINCT s.ip) FROM sessions s WHERE {s}"))?,
        "downloads": q.count(&format!("SELECT count(*) FROM downloads d WHERE {d}"))?,
        "success_pct": q.db.query_one(&format!(
            "SELECT coalesce(round(sum(CASE WHEN a.success THEN 1 ELSE 0 END)::numeric
                                   / NULLIF(count(*), 0) * 100, 2), 0)::float8
             FROM auth a WHERE {a}"), &[&q.since])?.get::<_, f64>(0),
        "unique_passwords": q.count(&format!("SELECT count(DISTINCT a.password) FROM auth a WHERE {a}"))?,
        "unique_usernames": q.count(&format!("SELECT count(DISTINCT a.username) FROM auth a WHERE {a}"))?,
        "unique_malware_hashes": q.count(&format!(
            "SELECT count(DISTINCT d.shasum) FROM downloads d WHERE {d} AND d.shasum IS NOT NULL"))?,
        "novel_passwords": count_lines(&wordlists.join(w.wordlist_period).join("novel_passwords.txt")),
    });

    let mut out = json!({
        "window": w.name,
        "generated_at": iso(now),
        "overview": overview,
        "top_usernames": q.rows(&format!(
            "SELECT a.username, count(*) AS attempts,
                    sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful
             FROM auth a WHERE {a} GROUP BY a.username ORDER BY attempts DESC LIMIT 15"))?,
        "top_passwords": q.rows(&format!(
            "SELECT a.password, count(*) AS attempts FROM auth a WHERE {a}
             GROUP BY a.password ORDER BY attempts DESC LIMIT 15"))?,
        "top_pairs": q.rows(&format!(
            "SELECT a.username, a.password, count(*) AS attempts FROM auth a WHERE {a}
             GROUP BY a.username, a.password ORDER BY attempts DESC LIMIT 10"))?,
        "timeseries": q.rows(&format!(
            "SELECT {} AS t,
                    sum(CASE WHEN a.success THEN 1 ELSE 0 END) AS successful,
                    sum(CASE WHEN NOT a.success THEN 1 ELSE 0 END) AS failed
             FROM auth a WHERE {a} GROUP BY 1 ORDER BY 1", w.bucket))?,
        "by_hour": q.rows(&format!(
            "SELECT LPAD(EXTRACT(HOUR FROM a.timestamp)::int::text, 2, '0') || ':00' AS hour,
                    EXTRACT(HOUR FROM a.timestamp)::int AS h, count(*) AS attempts
             FROM auth a WHERE {a} GROUP BY hour, h ORDER BY h"))?,
        "by_dow": q.rows(&format!(
            "SELECT TO_CHAR(a.timestamp, 'Dy') AS day,
                    EXTRACT(DOW FROM a.timestamp)::int AS dow, count(*) AS attempts
             FROM auth a WHERE {a} GROUP BY day, dow ORDER BY dow"))?,
        "ssh_clients": q.rows(&format!(
            "SELECT c.version AS client_version, count(*) AS connections
             FROM sessions s JOIN clients c ON s.client = c.id
             WHERE {s} GROUP BY c.version ORDER BY connections DESC LIMIT 10"))?,
        "top_urls": q.rows(&format!(
            "SELECT d.url, count(*) AS downloads FROM downloads d
             WHERE {d} AND d.url IS NOT NULL GROUP BY d.url ORDER BY downloads DESC LIMIT 10"))?,
        "cmd_log": q.rows(&format!(
            "SELECT i.timestamp AS time, s.ip, left(i.session, 8) AS session, i.input
             FROM input i JOIN sessions s ON i.session = s.id
             WHERE {i} ORDER BY i.timestamp DESC LIMIT 100"))?,
        "auth_log": q.rows(&format!(
            "SELECT a.timestamp AS time, s.ip, a.username, a.password, a.success,
                    left(a.session, 8) AS session
             FROM auth a JOIN sessions s ON a.session = s.id
             WHERE {a} ORDER BY a.timestamp DESC LIMIT 100"))?,
        "dl_log": q.rows(&format!(
            "SELECT d.timestamp AS time, s.ip, left(d.session, 8) AS session, d.url, d.source, d.outfile, d.shasum
             FROM downloads d JOIN sessions s ON d.session = s.id
             WHERE {d} ORDER BY d.timestamp DESC LIMIT 50"))?,
        "malware_hashes_detail": q.rows(&format!(
            "SELECT d.shasum, d.url, d.source, count(*) AS downloads, min(d.timestamp) AS first_seen
             FROM downloads d WHERE {d} AND d.shasum IS NOT NULL
             GROUP BY d.shasum, d.url, d.source ORDER BY downloads DESC LIMIT 20"))?,
    });

    out["telnet"] = or_fallback(
        "telnet",
        compute_telnet(&mut q, w),
        json!({"overview": {}, "timeseries": [], "top_commands": [], "top_ips": [],
               "top_usernames": [], "top_passwords": [], "session_log": [], "auth_log": []}),
    );
    out["geo"] = or_fallback(
        "geo",
        compute_geo(&mut q),
        json!({"top_countries": [], "top_asns": [], "top_ips": [], "new_asns": [], "coverage_pct": 0.0}),
    );
    out["campaigns"] = if CAMPAIGN_WINDOWS.contains(&w.name) {
        json!(campaigns)
    } else {
        json!([])
    };
    Ok(out)
}

fn write_json(dir: &Path, window: &str, data: &Value) -> Result<()> {
    write_atomic(
        &dir.join(format!("{window}.json")),
        &serde_json::to_vec(data)?,
    )?;
    Ok(())
}

fn main() -> ExitCode {
    let stats_dir = PathBuf::from(env_or("STATS_DIR", "/stats"));
    let wordlists = PathBuf::from(env_or("WORDLIST_DIR", "/wordlists"));

    println!("[{}] Connecting...", ts());
    let mut db = match db::connect(&HashMap::default(), &HOST_PORT_DEFAULTS) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[{}] ERROR: {e:#}", ts());
            return ExitCode::FAILURE;
        }
    };

    let readers = geoip::GeoReaders::open();
    if let Err(e) = geoip::enrich_new_ips(&mut db, &readers) {
        eprintln!("WARNING: geo enrichment failed: {e:#}");
    }

    println!("[{}] Running campaign detection...", ts());
    let campaigns = campaign::detect_campaign(&mut db)
        .and_then(|()| campaign::active_campaigns(&mut db))
        .unwrap_or_else(|e| {
            eprintln!("WARNING: campaign detection failed: {e:#}");
            Vec::new()
        });
    println!(
        "[{}] Campaign detection done ({} active)",
        ts(),
        campaigns.len()
    );

    let mut failed = false;
    for w in &WINDOWS {
        match compute(&mut db, w, &wordlists, &campaigns)
            .and_then(|d| write_json(&stats_dir, w.name, &d).map(|()| d))
        {
            Ok(d) => println!(
                "[{}] {}: done ({} auth rows)",
                ts(),
                w.name,
                d["overview"]["auth_attempts"]
            ),
            Err(e) => {
                eprintln!("[{}] {}: ERROR: {e:#}", ts(), w.name);
                failed = true;
            }
        }
    }

    println!("[{}] Done.", ts());
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
