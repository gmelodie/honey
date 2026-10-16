//! Groups Cowrie JSON log events into rows for the `PostgreSQL` tables.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub sensor: String,
    pub ip: Option<String>,
    pub starttime: Option<String>,
    pub endtime: Option<String>,
    pub termsize: String,
    pub client: Option<String>,
    pub protocol: String,
}

#[derive(Debug, PartialEq)]
pub struct Auth {
    pub session: Option<String>,
    pub success: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Input {
    pub session: Option<String>,
    pub timestamp: Option<String>,
    pub realm: String,
    pub input: String,
    pub success: bool,
}

#[derive(Debug, PartialEq)]
pub struct Download {
    pub session: Option<String>,
    pub timestamp: Option<String>,
    pub url: Option<String>,
    pub outfile: String,
    pub shasum: String,
}

#[derive(Debug, PartialEq)]
pub struct Fingerprint {
    pub session: Option<String>,
    pub username: Option<String>,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Default)]
pub struct Batch {
    pub sensors: Vec<String>,
    pub clients: Vec<String>,
    pub sessions: Vec<Session>,
    pub auth: Vec<Auth>,
    pub input: Vec<Input>,
    pub ttylogs: Vec<(String, String)>,
    pub downloads: Vec<Download>,
    pub fingerprints: Vec<Fingerprint>,
}

/// `PostgreSQL` rejects NUL bytes in text columns.
fn text(ev: &Value, key: &str) -> Option<String> {
    ev.get(key)?.as_str().map(|s| s.replace('\0', ""))
}

fn text_or(ev: &Value, key: &str, default: &str) -> String {
    text(ev, key).unwrap_or_else(|| default.to_string())
}

fn scalar(ev: &Value, key: &str) -> String {
    match ev.get(key) {
        Some(Value::String(s)) => s.replace('\0', ""),
        Some(Value::Null) | None => String::new(),
        Some(v) => v.to_string(),
    }
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn push_unique(seen: &mut HashSet<String>, list: &mut Vec<String>, item: String) {
    if seen.insert(item.clone()) {
        list.push(item);
    }
}

impl Batch {
    pub fn collect(events: impl IntoIterator<Item = Value>) -> Self {
        let mut b = Self::default();
        let mut sensors = HashSet::new();
        let mut clients = HashSet::new();
        let mut session_idx: HashMap<String, usize> = HashMap::new();
        let mut ttylogs = HashSet::new();

        for ev in events {
            let eid = text_or(&ev, "eventid", "");
            let sid = text(&ev, "session").filter(|s| !s.is_empty());
            let sensor = text_or(&ev, "sensor", "cowrie");
            let ts = text(&ev, "timestamp");
            let src_ip = text(&ev, "src_ip");

            push_unique(&mut sensors, &mut b.sensors, sensor.clone());

            let s = sid.as_ref().map(|id| {
                *session_idx.entry(id.clone()).or_insert_with(|| {
                    b.sessions.push(Session {
                        id: id.clone(),
                        sensor: sensor.clone(),
                        ip: src_ip.clone(),
                        protocol: "ssh".to_string(),
                        ..Session::default()
                    });
                    b.sessions.len() - 1
                })
            });

            match eid.as_str() {
                "cowrie.session.connect" => {
                    if let Some(s) = s.map(|i| &mut b.sessions[i]) {
                        s.starttime = ts;
                        s.ip = src_ip;
                        if let Some(p) = text(&ev, "protocol") {
                            s.protocol = p;
                        }
                    }
                }
                "cowrie.session.closed" => {
                    if let Some(s) = s.map(|i| &mut b.sessions[i]) {
                        s.endtime = ts;
                    }
                }
                "cowrie.client.version" => {
                    let version = truncate(&text_or(&ev, "version", ""), 50);
                    if let Some(s) = s.map(|i| &mut b.sessions[i]) {
                        s.client = Some(version.clone());
                    }
                    push_unique(&mut clients, &mut b.clients, version);
                }
                "cowrie.client.size" => {
                    if let Some(s) = s.map(|i| &mut b.sessions[i]) {
                        let size = format!("{}x{}", scalar(&ev, "width"), scalar(&ev, "height"));
                        s.termsize = truncate(&size, 7);
                    }
                }
                "cowrie.login.success" | "cowrie.login.failed" => b.auth.push(Auth {
                    session: sid,
                    success: eid == "cowrie.login.success",
                    username: text(&ev, "username"),
                    password: text(&ev, "password"),
                    timestamp: ts,
                }),
                "cowrie.command.input" => b.input.push(Input {
                    session: sid,
                    timestamp: ts,
                    realm: truncate(&text_or(&ev, "realm", ""), 20),
                    input: text_or(&ev, "input", ""),
                    success: ev.get("success").and_then(Value::as_bool).unwrap_or(false),
                }),
                "cowrie.log.open" | "cowrie.log.closed" => {
                    let ttylog = text(&ev, "ttylog")
                        .filter(|t| !t.is_empty())
                        .or_else(|| text(&ev, "filename"))
                        .filter(|t| !t.is_empty());
                    if let (Some(sid), Some(ttylog)) = (sid, ttylog) {
                        let key = (sid, ttylog);
                        if ttylogs.insert(key.clone()) {
                            b.ttylogs.push(key);
                        }
                    }
                }
                "cowrie.session.file_download" => b.downloads.push(Download {
                    session: sid,
                    timestamp: ts,
                    url: text(&ev, "url").filter(|u| !u.is_empty()),
                    outfile: text_or(&ev, "outfile", ""),
                    shasum: text_or(&ev, "shasum", ""),
                }),
                "cowrie.client.fingerprint" => b.fingerprints.push(Fingerprint {
                    session: sid,
                    username: text(&ev, "username"),
                    fingerprint: text(&ev, "fingerprint"),
                }),
                _ => {}
            }
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn collect(evs: Vec<Value>) -> Batch {
        Batch::collect(evs)
    }

    #[test]
    fn empty_input_yields_empty_batch() {
        let b = collect(vec![]);
        assert!(b.sensors.is_empty() && b.sessions.is_empty() && b.auth.is_empty());
    }

    #[test]
    fn connect_creates_session_with_default_protocol_and_sensor() {
        let b = collect(vec![json!({
            "eventid": "cowrie.session.connect", "session": "s1",
            "src_ip": "1.2.3.4", "timestamp": "t0"
        })]);
        assert_eq!(b.sensors, ["cowrie"]);
        assert_eq!(
            b.sessions,
            [Session {
                id: "s1".into(),
                sensor: "cowrie".into(),
                ip: Some("1.2.3.4".into()),
                starttime: Some("t0".into()),
                protocol: "ssh".into(),
                ..Session::default()
            }]
        );
    }

    #[test]
    fn connect_protocol_overrides_and_later_events_do_not_duplicate_session() {
        let b = collect(vec![
            json!({"eventid": "cowrie.session.connect", "session": "s1", "protocol": "telnet", "sensor": "hp2"}),
            json!({"eventid": "cowrie.session.closed", "session": "s1", "timestamp": "t9"}),
            json!({"eventid": "cowrie.client.size", "session": "s1", "width": 80, "height": 24}),
        ]);
        assert_eq!(b.sessions.len(), 1);
        let s = &b.sessions[0];
        assert_eq!((s.protocol.as_str(), s.sensor.as_str()), ("telnet", "hp2"));
        assert_eq!(s.endtime.as_deref(), Some("t9"));
        assert_eq!(s.termsize, "80x24");
        assert_eq!(b.sensors, ["hp2", "cowrie"]);
    }

    #[test]
    fn session_first_created_by_non_connect_event_keeps_its_ip() {
        let b = collect(vec![
            json!({"eventid": "cowrie.session.closed", "session": "s", "src_ip": "9.9.9.9"}),
        ]);
        assert_eq!(b.sessions[0].ip.as_deref(), Some("9.9.9.9"));
        assert_eq!(b.sessions[0].starttime, None);
    }

    #[test]
    fn empty_or_missing_session_id_creates_no_session() {
        let b = collect(vec![
            json!({"eventid": "cowrie.session.connect", "session": ""}),
            json!({"eventid": "cowrie.login.failed", "username": "u", "password": "p"}),
        ]);
        assert!(b.sessions.is_empty());
        assert_eq!(b.auth.len(), 1);
        assert_eq!(b.auth[0].session, None);
    }

    #[test]
    fn nul_bytes_are_stripped_from_every_text_field() {
        let b = collect(vec![
            json!({"eventid": "cowrie.login.failed", "session": "s\u{0}1", "username": "ro\u{0}ot", "password": "\u{0}pw"}),
            json!({"eventid": "cowrie.command.input", "session": "s1", "input": "ls\u{0} -l", "realm": "r\u{0}"}),
            json!({"eventid": "cowrie.client.version", "session": "s1", "version": "SSH\u{0}-2"}),
            json!({"eventid": "cowrie.log.open", "session": "s1", "ttylog": "a\u{0}b"}),
            json!({"eventid": "cowrie.client.size", "session": "s1", "width": "8\u{0}0", "height": "2"}),
        ]);
        assert_eq!(b.auth[0].session.as_deref(), Some("s1"));
        assert_eq!(b.auth[0].username.as_deref(), Some("root"));
        assert_eq!(b.auth[0].password.as_deref(), Some("pw"));
        assert_eq!(
            (b.input[0].input.as_str(), b.input[0].realm.as_str()),
            ("ls -l", "r")
        );
        assert_eq!(b.clients, ["SSH-2"]);
        assert_eq!(b.ttylogs, [("s1".to_string(), "ab".to_string())]);
        assert_eq!(b.sessions[0].termsize, "80x2");
        assert_eq!(b.sessions.len(), 1);
    }

    #[test]
    fn login_events_set_success_flag() {
        let b = collect(vec![
            json!({"eventid": "cowrie.login.success", "session": "s", "username": "a", "password": "b", "timestamp": "t"}),
            json!({"eventid": "cowrie.login.failed", "session": "s"}),
        ]);
        assert_eq!(
            b.auth,
            [
                Auth {
                    session: Some("s".into()),
                    success: true,
                    username: Some("a".into()),
                    password: Some("b".into()),
                    timestamp: Some("t".into())
                },
                Auth {
                    session: Some("s".into()),
                    success: false,
                    username: None,
                    password: None,
                    timestamp: None
                },
            ]
        );
    }

    #[test]
    fn version_truncates_by_chars_not_bytes_and_dedupes_clients() {
        let long = "ü".repeat(60);
        let b = collect(vec![
            json!({"eventid": "cowrie.client.version", "session": "a", "version": long}),
            json!({"eventid": "cowrie.client.version", "session": "b", "version": long}),
        ]);
        assert_eq!(b.clients, ["ü".repeat(50)]);
        assert_eq!(
            b.sessions[0].client.as_deref(),
            Some("ü".repeat(50).as_str())
        );
    }

    #[test]
    fn missing_version_records_empty_client() {
        let b = collect(vec![
            json!({"eventid": "cowrie.client.version", "session": "a"}),
        ]);
        assert_eq!(b.clients, [""]);
        assert_eq!(b.sessions[0].client.as_deref(), Some(""));
    }

    #[test]
    fn realm_truncates_to_20_chars_and_success_defaults_false() {
        let b = collect(vec![
            json!({"eventid": "cowrie.command.input", "session": "a", "input": "id", "realm": "é".repeat(25), "success": true}),
            json!({"eventid": "cowrie.command.input", "session": "a", "input": "id", "realm": "x".repeat(20)}),
            json!({"eventid": "cowrie.command.input", "session": "a", "success": "yes"}),
        ]);
        assert_eq!(b.input[0].realm, "é".repeat(20));
        assert!(b.input[0].success);
        assert_eq!(b.input[1].realm, "x".repeat(20));
        assert!(!b.input[1].success);
        assert_eq!(
            (
                b.input[2].input.as_str(),
                b.input[2].realm.as_str(),
                b.input[2].success
            ),
            ("", "", false)
        );
    }

    #[test]
    fn ttylog_dedupes_per_session_and_falls_back_to_filename() {
        let b = collect(vec![
            json!({"eventid": "cowrie.log.open", "session": "a", "ttylog": "t1"}),
            json!({"eventid": "cowrie.log.closed", "session": "a", "ttylog": "t1"}),
            json!({"eventid": "cowrie.log.closed", "session": "b", "ttylog": "t1"}),
            json!({"eventid": "cowrie.log.open", "session": "a", "ttylog": "", "filename": "f1"}),
            json!({"eventid": "cowrie.log.open", "session": "a", "ttylog": "", "filename": ""}),
            json!({"eventid": "cowrie.log.open", "ttylog": "orphan"}),
        ]);
        let pair = |s: &str, t: &str| (s.to_string(), t.to_string());
        assert_eq!(
            b.ttylogs,
            [pair("a", "t1"), pair("b", "t1"), pair("a", "f1")]
        );
    }

    #[test]
    fn download_empty_url_becomes_none_and_missing_fields_are_empty() {
        let b = collect(vec![
            json!({"eventid": "cowrie.session.file_download", "session": "a", "url": "", "outfile": "o", "shasum": "h"}),
            json!({"eventid": "cowrie.session.file_download", "session": "a", "url": "http://x/y"}),
        ]);
        assert_eq!(b.downloads[0].url, None);
        assert_eq!(
            (
                b.downloads[0].outfile.as_str(),
                b.downloads[0].shasum.as_str()
            ),
            ("o", "h")
        );
        assert_eq!(b.downloads[1].url.as_deref(), Some("http://x/y"));
        assert_eq!(
            (
                b.downloads[1].outfile.as_str(),
                b.downloads[1].shasum.as_str()
            ),
            ("", "")
        );
    }

    #[test]
    fn fingerprint_event_is_recorded() {
        let b = collect(vec![
            json!({"eventid": "cowrie.client.fingerprint", "session": "a", "username": "u", "fingerprint": "ab:cd"}),
        ]);
        assert_eq!(
            b.fingerprints,
            [Fingerprint {
                session: Some("a".into()),
                username: Some("u".into()),
                fingerprint: Some("ab:cd".into())
            }]
        );
    }

    #[test]
    fn size_without_dimensions_is_bare_x_and_null_width_is_empty() {
        let b = collect(vec![
            json!({"eventid": "cowrie.client.size", "session": "a"}),
            json!({"eventid": "cowrie.client.size", "session": "b", "width": null, "height": 5}),
        ]);
        assert_eq!(b.sessions[0].termsize, "x");
        assert_eq!(b.sessions[1].termsize, "x5");
    }

    #[test]
    fn unknown_event_still_registers_sensor_and_session() {
        let b = collect(vec![
            json!({"eventid": "cowrie.other", "session": "a", "sensor": "s9"}),
        ]);
        assert_eq!(
            (b.sensors.as_slice(), b.sessions.len()),
            (["s9".to_string()].as_slice(), 1)
        );
    }
}
