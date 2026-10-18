//! Tails the Cowrie JSON log and fans out a trimmed view of each event.

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Map, Value, json};
use tokio::sync::broadcast;

const BACKLOG: usize = 200;
/// How far back from the end the first open of the log reads to fill the backlog.
const BACKFILL_BYTES: u64 = 256 * 1024;
const POLL: Duration = Duration::from_millis(250);
const MAX_FIELD: usize = 512;
const MAX_SESSIONS: usize = 50_000;

pub type Line = Arc<str>;

pub struct Live {
    inner: Mutex<Inner>,
}

struct Inner {
    backlog: VecDeque<Line>,
    tx: Option<broadcast::Sender<Line>>,
}

impl Live {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                backlog: VecDeque::with_capacity(BACKLOG),
                tx: Some(broadcast::channel(1024).0),
            }),
        })
    }

    pub fn spawn(path: PathBuf) -> Arc<Self> {
        let live = Self::new();
        let tail_live = live.clone();
        std::thread::spawn(move || tail(&tail_live, &path));
        live
    }

    /// Backlog and receiver come from one critical section, so no line is lost or repeated.
    pub fn subscribe(&self) -> (Vec<Line>, Option<broadcast::Receiver<Line>>) {
        let inner = self.inner.lock().expect("live lock");
        (
            inner.backlog.iter().cloned().collect(),
            inner.tx.as_ref().map(broadcast::Sender::subscribe),
        )
    }

    /// Ends every open stream so graceful shutdown does not wait on them.
    pub fn close(&self) {
        self.inner.lock().expect("live lock").tx = None;
    }

    fn publish(&self, line: Line) {
        let mut inner = self.inner.lock().expect("live lock");
        if inner.backlog.len() == BACKLOG {
            inner.backlog.pop_front();
        }
        inner.backlog.push_back(line.clone());
        if let Some(tx) = &inner.tx {
            let _ = tx.send(line);
        }
    }
}

fn tail(live: &Live, path: &Path) {
    let mut protos = HashMap::new();
    let mut first = true;
    loop {
        match open_at(path, first) {
            Ok((reader, ino)) => {
                first = false;
                follow(live, reader, path, ino, &mut protos);
            }
            Err(_) => std::thread::sleep(POLL * 4),
        }
    }
}

/// The first open starts near the end, skipping the partial line; a reopen after rotation starts at 0.
fn open_at(path: &Path, backfill: bool) -> std::io::Result<(BufReader<File>, u64)> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    let start = if backfill {
        meta.len().saturating_sub(BACKFILL_BYTES)
    } else {
        0
    };
    file.seek(SeekFrom::Start(start))?;

    let mut reader = BufReader::new(file);
    if start > 0 {
        reader.skip_until(b'\n')?;
    }
    Ok((reader, meta.ino()))
}

fn follow(
    live: &Live,
    mut reader: BufReader<File>,
    path: &Path,
    ino: u64,
    protos: &mut HashMap<String, String>,
) {
    let mut buf = Vec::new();
    loop {
        drain(live, &mut reader, &mut buf, protos);
        std::thread::sleep(POLL);

        let pos = reader.stream_position().unwrap_or(0);
        let rotated = std::fs::metadata(path).map_or(true, |m| m.ino() != ino || m.len() < pos);
        if rotated {
            drain(live, &mut reader, &mut buf, protos);
            return;
        }
    }
}

/// Reads complete lines to EOF; a trailing partial line stays in `buf` for the next call.
fn drain(
    live: &Live,
    reader: &mut BufReader<File>,
    buf: &mut Vec<u8>,
    protos: &mut HashMap<String, String>,
) {
    loop {
        match reader.read_until(b'\n', buf) {
            Ok(0) | Err(_) => return,
            Ok(_) if buf.last() != Some(&b'\n') => return,
            Ok(_) => {}
        }

        if let Ok(ev) = serde_json::from_slice::<Value>(buf)
            && let Some(line) = render(&ev, protos)
        {
            live.publish(line.into());
        }
        buf.clear();
    }
}

fn field(ev: &Value, key: &str) -> Value {
    match ev.get(key) {
        Some(Value::String(s)) => Value::String(s.chars().take(MAX_FIELD).collect()),
        Some(v @ (Value::Number(_) | Value::Bool(_))) => v.clone(),
        _ => Value::Null,
    }
}

/// Cowrie sets `protocol` only on connect; `protos` carries it to later events.
pub fn render(ev: &Value, protos: &mut HashMap<String, String>) -> Option<String> {
    let session = ev.get("session")?.as_str()?;
    let (kind, extra) = match ev.get("eventid")?.as_str()? {
        "cowrie.session.connect" => {
            if protos.len() >= MAX_SESSIONS {
                protos.clear();
            }
            let proto = ev.get("protocol").and_then(Value::as_str).unwrap_or("");
            protos.insert(session.to_string(), proto.to_string());
            ("connect", json!({}))
        }
        "cowrie.client.version" => ("client", json!({"v": field(ev, "version")})),
        "cowrie.login.success" | "cowrie.login.failed" => (
            "login",
            json!({
                "ok": ev["eventid"] == "cowrie.login.success",
                "u": field(ev, "username"),
                "pw": field(ev, "password"),
            }),
        ),
        "cowrie.command.input" => ("cmd", json!({"in": field(ev, "input")})),
        "cowrie.session.file_download" => ("dl", json!({"url": field(ev, "url")})),
        "cowrie.session.file_upload" => ("ul", json!({"f": field(ev, "filename")})),
        "cowrie.session.closed" => ("closed", json!({"dur": field(ev, "duration")})),
        _ => return None,
    };

    let proto = match kind {
        "closed" => protos.remove(session),
        _ => protos.get(session).cloned(),
    };
    let mut out = Map::new();
    out.insert("t".into(), field(ev, "timestamp"));
    out.insert("p".into(), proto.map_or(Value::Null, Value::String));
    out.insert("ip".into(), field(ev, "src_ip"));
    out.insert("s".into(), field(ev, "session"));
    out.insert("k".into(), kind.into());
    if let Value::Object(extra) = extra {
        out.extend(extra);
    }
    Some(Value::Object(out).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(ev: &Value, protos: &mut HashMap<String, String>) -> Option<Value> {
        render(ev, protos).map(|s| serde_json::from_str(&s).unwrap())
    }

    #[test]
    fn protocol_from_connect_follows_the_session_until_closed() {
        let mut p = HashMap::new();
        let c = r(
            &json!({"eventid": "cowrie.session.connect", "session": "s1", "protocol": "telnet",
                    "src_ip": "1.2.3.4", "timestamp": "t0", "sensor": "secret"}),
            &mut p,
        )
        .unwrap();
        assert_eq!(
            c,
            json!({"t": "t0", "p": "telnet", "ip": "1.2.3.4", "s": "s1", "k": "connect"})
        );

        let l = r(
            &json!({"eventid": "cowrie.login.failed", "session": "s1", "username": "root", "password": "x"}),
            &mut p,
        )
        .unwrap();
        assert_eq!(
            (&l["p"], &l["ok"], &l["u"], &l["pw"]),
            (&json!("telnet"), &json!(false), &json!("root"), &json!("x"))
        );

        let z = r(
            &json!({"eventid": "cowrie.session.closed", "session": "s1", "duration": 1.5}),
            &mut p,
        )
        .unwrap();
        assert_eq!((&z["p"], &z["dur"]), (&json!("telnet"), &json!(1.5)));
        assert!(p.is_empty());
    }

    #[test]
    fn unknown_events_and_sessionless_events_are_dropped() {
        let mut p = HashMap::new();
        assert!(
            r(
                &json!({"eventid": "cowrie.client.size", "session": "s"}),
                &mut p
            )
            .is_none()
        );
        assert!(
            r(
                &json!({"eventid": "cowrie.command.input", "input": "id"}),
                &mut p
            )
            .is_none()
        );
        assert!(r(&json!({"session": "s"}), &mut p).is_none());
    }

    #[test]
    fn long_fields_are_cut_and_unseen_session_has_null_protocol() {
        let mut p = HashMap::new();
        let c = r(
            &json!({"eventid": "cowrie.command.input", "session": "s", "input": "é".repeat(2000)}),
            &mut p,
        )
        .unwrap();
        assert_eq!(c["in"].as_str().unwrap().chars().count(), MAX_FIELD);
        assert_eq!(c["p"], Value::Null);
    }

    #[test]
    fn backlog_keeps_the_newest_lines() {
        let live = Live::new();
        for i in 0..BACKLOG + 5 {
            live.publish(i.to_string().into());
        }
        let (backlog, rx) = live.subscribe();
        assert_eq!(backlog.len(), BACKLOG);
        assert_eq!(&*backlog[0], "5");
        assert!(rx.is_some());

        live.close();
        assert!(live.subscribe().1.is_none());
    }
}
