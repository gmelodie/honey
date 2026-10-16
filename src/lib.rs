pub mod bloom;
pub mod cowrie;
pub mod db;
pub mod rules;
pub mod trends;
pub mod web;

use chrono::{DateTime, SecondsFormat, Utc};

pub fn ts() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn iso(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Micros, false)
}

/// Writes through a temp file and rename, so readers never see a partial file.
pub fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)
}

pub fn gz_path(path: &std::path::Path) -> std::path::PathBuf {
    let mut p = path.as_os_str().to_owned();
    p.push(".gz");
    p.into()
}

pub fn gzip(data: &[u8]) -> std::io::Result<Vec<u8>> {
    use std::io::Write;

    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(data)?;
    gz.finish()
}

pub fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn write_atomic_creates_parents_overwrites_and_leaves_no_tmp() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a/b/f.txt");
        write_atomic(&p, b"one").unwrap();
        write_atomic(&p, b"2").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"2");
        let names: Vec<_> = std::fs::read_dir(p.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, ["f.txt"]);
    }

    #[test]
    fn write_atomic_overwrites_stale_tmp_from_a_crash() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("f");
        std::fs::write(d.path().join("f.tmp"), b"stale-longer-content").unwrap();
        write_atomic(&p, b"new").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"new");
        assert!(!d.path().join("f.tmp").exists());
    }

    #[test]
    fn write_atomic_fails_when_target_is_a_directory() {
        let d = tempfile::tempdir().unwrap();
        assert!(write_atomic(d.path(), b"x").is_err());
    }

    #[test]
    fn iso_uses_micros_and_plus_zero_offset() {
        let t = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(iso(t), "2024-01-02T03:04:05.000000+00:00");
    }

    #[test]
    fn env_or_returns_default_when_unset() {
        assert_eq!(env_or("HONEY_TEST_SURELY_UNSET_VAR", "dflt"), "dflt");
    }
}
