use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use postgres::types::Type;
use postgres::{Client, NoTls, Row};
use serde_json::{Map, Value, json};

/// Reads `POSTGRES_*` from the environment, then from `fallback`, then from `defaults`.
pub fn connect(fallback: &HashMap<String, String>, defaults: &[(&str, &str)]) -> Result<Client> {
    let get = |key: &str| -> Result<String> {
        std::env::var(key)
            .ok()
            .or_else(|| fallback.get(key).cloned())
            .or_else(|| {
                defaults
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| v.to_string())
            })
            .with_context(|| format!("{key} is not set"))
    };

    let mut cfg = postgres::Config::new();
    cfg.host(&get("POSTGRES_HOST")?)
        .port(get("POSTGRES_PORT")?.parse().context("POSTGRES_PORT")?)
        .dbname(&get("POSTGRES_DB")?)
        .user(&get("POSTGRES_USER")?)
        .password(get("POSTGRES_PASSWORD")?);
    Ok(cfg.connect(NoTls)?)
}

pub const HOST_PORT_DEFAULTS: [(&str, &str); 2] =
    [("POSTGRES_HOST", "127.0.0.1"), ("POSTGRES_PORT", "5432")];

/// Parses `KEY=VALUE` lines; strips matching single or double quotes.
pub fn parse_dotenv(text: &str) -> HashMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| {
            let v = v.trim();
            let unquoted = ['"', '\'']
                .iter()
                .find_map(|q| v.strip_prefix(*q).and_then(|v| v.strip_suffix(*q)))
                .unwrap_or(v);
            (k.trim().to_string(), unquoted.to_string())
        })
        .collect()
}

pub fn row_json(row: &Row) -> Result<Value> {
    let mut obj = Map::new();
    for (i, col) in row.columns().iter().enumerate() {
        let v = match *col.type_() {
            Type::INT8 => json!(row.try_get::<_, Option<i64>>(i)?),
            Type::INT4 => json!(row.try_get::<_, Option<i32>>(i)?),
            Type::FLOAT8 => json!(row.try_get::<_, Option<f64>>(i)?),
            Type::BOOL => json!(row.try_get::<_, Option<bool>>(i)?),
            Type::TIMESTAMPTZ => json!(row.try_get::<_, Option<DateTime<Utc>>>(i)?.map(crate::iso)),
            Type::JSON | Type::JSONB => row.try_get::<_, Option<Value>>(i)?.unwrap_or(Value::Null),
            Type::TEXT | Type::VARCHAR | Type::BPCHAR => {
                json!(row.try_get::<_, Option<String>>(i)?)
            }
            ref t => bail!("column {}: unsupported type {t}", col.name()),
        };
        obj.insert(col.name().to_string(), v);
    }
    Ok(Value::Object(obj))
}

pub fn rows_json(rows: &[Row]) -> Result<Vec<Value>> {
    rows.iter().map(row_json).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotenv_parses_pairs_quotes_comments_and_blanks() {
        let m =
            parse_dotenv("# c\n\nA=1\n  B = \"two words\" \nC='x'\nD=\nE=a=b\nnoequals\n  #X=1\n");
        assert_eq!(m["A"], "1");
        assert_eq!(m["B"], "two words");
        assert_eq!(m["C"], "x");
        assert_eq!(m["D"], "");
        assert_eq!(m["E"], "a=b");
        assert_eq!(m.len(), 5);
    }

    #[test]
    fn dotenv_keeps_mismatched_or_lone_quotes() {
        let m = parse_dotenv("A=\"x'\nB=\"\nC=''\nD=\"\"\r\nE='\"q\"'");
        assert_eq!(m["A"], "\"x'");
        assert_eq!(m["B"], "\"");
        assert_eq!(m["C"], "");
        assert_eq!(m["D"], "");
        assert_eq!(m["E"], "\"q\"");
    }

    #[test]
    fn dotenv_last_duplicate_wins() {
        assert_eq!(parse_dotenv("A=1\nA=2")["A"], "2");
    }
}
