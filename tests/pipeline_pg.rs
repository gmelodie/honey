use std::path::Path;
use std::process::Command;

use chrono::{Duration, Utc};
use honey::bloom::ScalableBloom;
use postgres::config::Host;
use postgres::{Client, Config, NoTls};
use serde_json::{Value, json};

const SCHEMA: &str = include_str!("../postgres/schema.sql");

/// The test truncates tables, so it only runs against a database whose name contains "test".
fn test_db() -> Option<(Config, Client)> {
    let url = std::env::var("HONEY_TEST_PG").ok()?;
    let cfg: Config = url.parse().unwrap();
    let db = cfg.get_dbname().unwrap_or_default();
    assert!(
        db.contains("test"),
        "refusing to truncate tables in database {db:?}"
    );
    let client = cfg.connect(NoTls).unwrap();
    Some((cfg, client))
}

fn pg_env(cfg: &Config) -> Vec<(&'static str, String)> {
    let Host::Tcp(host) = &cfg.get_hosts()[0] else {
        panic!("HONEY_TEST_PG needs a TCP host")
    };
    let password = cfg
        .get_password()
        .map(|p| String::from_utf8_lossy(p).into_owned());
    vec![
        ("POSTGRES_HOST", host.clone()),
        (
            "POSTGRES_PORT",
            cfg.get_ports().first().copied().unwrap_or(5432).to_string(),
        ),
        ("POSTGRES_DB", cfg.get_dbname().unwrap().to_string()),
        ("POSTGRES_USER", cfg.get_user().unwrap().to_string()),
        ("POSTGRES_PASSWORD", password.unwrap_or_default()),
    ]
}

fn run(bin: &str, cfg: &Config, cwd: &Path, env: &[(&str, &Path)], args: &[&Path]) {
    let out = Command::new(bin)
        .current_dir(cwd)
        .envs(pg_env(cfg))
        .envs(env.iter().copied())
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{bin}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn cowrie_log() -> String {
    let t = |min: i64| {
        (Utc::now() - Duration::minutes(min))
            .format("%Y-%m-%dT%H:%M:%S%.6fZ")
            .to_string()
    };
    let (s1, s2) = ("a".repeat(32), "b".repeat(32));
    let events = [
        json!({"eventid": "cowrie.session.connect", "session": s1, "src_ip": "198.51.100.1", "timestamp": t(30), "protocol": "ssh"}),
        json!({"eventid": "cowrie.client.version", "session": s1, "version": "SSH-2.0-Go", "timestamp": t(30)}),
        json!({"eventid": "cowrie.client.size", "session": s1, "width": 4_294_967_295_u64, "height": 24, "timestamp": t(30)}),
        json!({"eventid": "cowrie.login.failed", "session": s1, "username": "root", "password": "123456", "timestamp": t(29)}),
        json!({"eventid": "cowrie.login.success", "session": s1, "username": "root", "password": "Zebra2024!", "timestamp": t(29)}),
        json!({"eventid": "cowrie.command.input", "session": s1, "input": "uname -a", "timestamp": t(28)}),
        json!({"eventid": "cowrie.session.file_download", "session": s1, "url": "http://x/b.sh", "outfile": "f", "shasum": "ab", "timestamp": t(28)}),
        json!({"eventid": "cowrie.session.closed", "session": s1, "timestamp": t(27)}),
        json!({"eventid": "cowrie.session.connect", "session": s2, "src_ip": "198.51.100.2", "timestamp": t(20), "protocol": "telnet"}),
        json!({"eventid": "cowrie.login.failed", "session": s2, "username": "admin", "password": "123456", "timestamp": t(19)}),
    ];
    events.iter().map(|e| e.to_string() + "\n").collect()
}

fn count(db: &mut Client, sql: &str) -> i64 {
    db.query_one(sql, &[]).unwrap().get(0)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn reset(db: &mut Client) {
    db.batch_execute(SCHEMA).unwrap();
    db.batch_execute(
        "TRUNCATE auth, input, ttylog, downloads, keyfingerprints, sessions, clients,
                  ip_geo_cache, campaign_events RESTART IDENTITY CASCADE",
    )
    .unwrap();
}

fn check_import(cfg: &Config, db: &mut Client, cwd: &Path, log: &Path) {
    let import = env!("CARGO_BIN_EXE_honey-import");
    run(import, cfg, cwd, &[], &[log]);
    assert_eq!(
        count(
            db,
            "SELECT count(*) FROM sessions WHERE protocol = 'telnet'"
        ),
        1
    );
    assert_eq!(
        count(
            db,
            "SELECT count(*) FROM sessions WHERE termsize = '4294967'"
        ),
        1
    );

    run(import, cfg, cwd, &[], &[log]);
    assert_eq!(count(db, "SELECT count(*) FROM sessions"), 2);
    assert_eq!(count(db, "SELECT count(*) FROM clients"), 1);
    assert_eq!(count(db, "SELECT count(*) FROM auth"), 6);
}

fn write_bloom(dir: &Path, words: &[&str]) {
    let mut bf = ScalableBloom::new(0.001);
    for w in words {
        bf.insert(w);
    }
    let mut data = Vec::new();
    bf.write_to(&mut data).unwrap();
    honey::write_atomic(&dir.join("reference.bloom"), &data).unwrap();
}

fn check_wordlists(words: &Path) {
    let all = words.join("alltime");
    assert_eq!(read(&all.join("passwords.txt")), "123456\nZebra2024!\n");
    assert_eq!(read(&all.join("usernames.txt")), "root\nadmin\n");
    assert_eq!(
        read(&all.join("passwords_usernames.txt")),
        "123456:admin\n123456:root\nZebra2024!:root\n"
    );
    assert_eq!(read(&all.join("novel_passwords.txt")), "Zebra2024!\n");
    assert!(read(&all.join("hashcat.rule")).contains("\n$2 $0 $2 $4 $!\nc $2 $0 $2 $4 $!\n"));
    assert_eq!(
        read(&words.join("daily/trending_passwords.txt")),
        "123456\nZebra2024!\n"
    );
    assert!(all.join("passwords.txt.gz").exists());
}

fn check_stats(stats: &Path) {
    for w in ["1h", "6h", "24h", "7d", "30d", "all"] {
        assert!(stats.join(format!("{w}.json")).exists(), "{w}.json missing");
    }

    let h1: Value = serde_json::from_str(&read(&stats.join("1h.json"))).unwrap();
    let o = &h1["overview"];
    assert_eq!(
        [&o["auth_attempts"], &o["connections"], &o["unique_ips"]],
        [&json!(6), &json!(2), &json!(2)]
    );
    assert_eq!(
        [&o["downloads"], &o["novel_passwords"], &o["success_pct"]],
        [&json!(2), &json!(1), &json!(33.33)]
    );
    assert_eq!(
        h1["top_passwords"][0],
        json!({"password": "123456", "attempts": 4})
    );
    assert_eq!(h1["telnet"]["overview"]["connections"], 1);
    assert_eq!(h1["telnet"]["overview"]["auth_attempts"], 2);
    assert_eq!(h1["campaigns"], json!([]));
}

#[test]
fn import_wordlists_and_stats_end_to_end() {
    let Some((cfg, mut db)) = test_db() else {
        return;
    };
    reset(&mut db);

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let (log, words, bloom, stats) = (
        root.join("cowrie.json"),
        root.join("w"),
        root.join("b"),
        root.join("s"),
    );
    std::fs::write(&log, cowrie_log()).unwrap();
    check_import(&cfg, &mut db, root, &log);

    write_bloom(&bloom, &["123456"]);
    let dirs: [(&str, &Path); 3] = [
        ("WORDLIST_DIR", &words),
        ("BLOOM_DIR", &bloom),
        ("STATS_DIR", &stats),
    ];
    run(
        env!("CARGO_BIN_EXE_honey-wordlists"),
        &cfg,
        root,
        &dirs,
        &[],
    );
    check_wordlists(&words);

    run(env!("CARGO_BIN_EXE_honey-stats"), &cfg, root, &dirs, &[]);
    check_stats(&stats);
}
