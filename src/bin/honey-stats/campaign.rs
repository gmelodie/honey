//! Campaign: a 2 h, 3-sigma auth spike with 30%+ of it from 3+ ASNs new since the 7-day baseline.

use std::collections::HashSet;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use honey::db::rows_json;
use postgres::Client;
use postgres::types::ToSql;
use serde_json::{Value, json};

use crate::round_to;

struct Spike {
    mean: f64,
    stdev: f64,
    attempts: i64,
    rate: f64,
    z: f64,
}

struct AsnActivity {
    asn: i32,
    asn_org: Option<String>,
    attempts: i64,
}

pub fn active_campaigns(db: &mut Client) -> Result<Vec<Value>> {
    rows_json(&db.query(
        "SELECT id, detected_at, onset_time, z_score::float8 AS z_score,
                spike_ratio::float8 AS spike_ratio, new_asn_count,
                peak_rate_per_hour::float8 AS peak_rate_per_hour,
                baseline_rate_per_hour::float8 AS baseline_rate_per_hour,
                new_asns, top_pairs, credential_pattern
         FROM campaign_events WHERE active = TRUE ORDER BY onset_time DESC",
        &[],
    )?)
}

fn mean_stdev(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    if xs.len() < 2 {
        return (mean, 0.0);
    }

    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, var.sqrt())
}

fn measure_spike(
    db: &mut Client,
    baseline_start: DateTime<Utc>,
    window_start: DateTime<Utc>,
) -> Result<Option<Spike>> {
    let hourly: Vec<f64> = db
        .query(
            "SELECT count(*) FROM auth a WHERE a.timestamp >= $1 AND a.timestamp < $2
             GROUP BY date_trunc('hour', a.timestamp)",
            &[&baseline_start, &window_start],
        )?
        .iter()
        .map(|r| r.get::<_, i64>(0) as f64)
        .collect();
    if hourly.len() < 12 {
        return Ok(None);
    }

    let (mean, stdev) = mean_stdev(&hourly);
    let attempts: i64 = db
        .query_one(
            "SELECT count(*) FROM auth a WHERE a.timestamp >= $1",
            &[&window_start],
        )?
        .get(0);
    let rate = attempts as f64 / 2.0;
    let z = (rate - mean) / stdev.max(1.0);
    Ok((z >= 3.0).then_some(Spike {
        mean,
        stdev,
        attempts,
        rate,
        z,
    }))
}

fn new_asn_activity(
    db: &mut Client,
    baseline_start: DateTime<Utc>,
    window_start: DateTime<Utc>,
) -> Result<Vec<AsnActivity>> {
    let baseline: HashSet<i32> = db
        .query(
            "SELECT DISTINCT g.asn FROM sessions s JOIN ip_geo_cache g ON g.ip = s.ip
             WHERE s.starttime >= $1 AND s.starttime < $2 AND g.asn IS NOT NULL",
            &[&baseline_start, &window_start],
        )?
        .iter()
        .map(|r| r.get(0))
        .collect();

    let mut asns: Vec<AsnActivity> = db
        .query(
            "SELECT g.asn, g.asn_org, count(a.id) FROM sessions s
             JOIN ip_geo_cache g ON g.ip = s.ip
             LEFT JOIN auth a ON a.session = s.id AND a.timestamp >= $1
             WHERE s.starttime >= $1 AND g.asn IS NOT NULL
             GROUP BY g.asn, g.asn_org",
            &[&window_start],
        )?
        .iter()
        .map(|r| AsnActivity {
            asn: r.get(0),
            asn_org: r.get(1),
            attempts: r.get(2),
        })
        .filter(|a| !baseline.contains(&a.asn))
        .collect();
    asns.sort_by_key(|a| std::cmp::Reverse(a.attempts));
    Ok(asns)
}

/// First 5-minute bucket in the last 6 h whose hourly rate passes mean + 2 sigma.
fn find_onset(db: &mut Client, now: DateTime<Utc>, spike: &Spike) -> Result<Option<DateTime<Utc>>> {
    let threshold = spike.mean + 2.0 * spike.stdev;
    Ok(db
        .query(
            "SELECT date_trunc('minute', a.timestamp)
                    - (EXTRACT(MINUTE FROM a.timestamp)::int % 5) * INTERVAL '1 minute' AS bucket,
                    count(*)
             FROM auth a WHERE a.timestamp >= $1 GROUP BY bucket ORDER BY bucket",
            &[&(now - Duration::hours(6))],
        )?
        .iter()
        .find(|r| (r.get::<_, i64>(1) * 12) as f64 > threshold)
        .map(|r| r.get(0)))
}

/// Top pairs from the new ASNs; `novel` marks pairs absent from the baseline top 10.
fn campaign_pairs(
    db: &mut Client,
    baseline_start: DateTime<Utc>,
    window_start: DateTime<Utc>,
    asns: &[AsnActivity],
) -> Result<Vec<Value>> {
    let baseline: HashSet<(Option<String>, Option<String>)> = db
        .query(
            "SELECT a.username, a.password FROM auth a
             WHERE a.timestamp >= $1 AND a.timestamp < $2
             GROUP BY a.username, a.password ORDER BY count(*) DESC LIMIT 10",
            &[&baseline_start, &window_start],
        )?
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect();

    let ids: Vec<i32> = asns.iter().map(|a| a.asn).collect();
    let rows = db.query(
        "SELECT a.username, a.password, count(*) FROM auth a
         JOIN sessions s ON a.session = s.id
         JOIN ip_geo_cache g ON g.ip = s.ip
         WHERE a.timestamp >= $1 AND g.asn = ANY($2)
         GROUP BY a.username, a.password ORDER BY count(*) DESC LIMIT 10",
        &[&window_start, &ids],
    )?;
    Ok(rows
        .iter()
        .map(|r| {
            let pair: (Option<String>, Option<String>) = (r.get(0), r.get(1));
            let novel = !baseline.contains(&pair);
            json!({"username": pair.0, "password": pair.1, "attempts": r.get::<_, i64>(2), "novel": novel})
        })
        .collect())
}

pub fn detect_campaign(db: &mut Client) -> Result<()> {
    let now = Utc::now();
    let window_start = now - Duration::hours(2);
    let baseline_start = now - Duration::days(7);

    db.execute(
        "UPDATE campaign_events SET active = FALSE WHERE onset_time < $1 AND active = TRUE",
        &[&(now - Duration::hours(48))],
    )?;

    let Some(spike) = measure_spike(db, baseline_start, window_start)? else {
        return Ok(());
    };
    let asns = new_asn_activity(db, baseline_start, window_start)?;
    let ratio = asns.iter().map(|a| a.attempts).sum::<i64>() as f64 / spike.attempts.max(1) as f64;
    if asns.len() < 3 || ratio < 0.30 {
        return Ok(());
    }

    let onset = find_onset(db, now, &spike)?.unwrap_or(window_start);
    let pairs = campaign_pairs(db, baseline_start, window_start, &asns)?;
    let novel = pairs.iter().filter(|p| p["novel"] == true).count();
    let pattern = if novel >= 2 { "novel" } else { "established" };
    let top_asns: Vec<Value> = asns
        .iter()
        .take(10)
        .map(|a| json!({"asn": a.asn, "asn_org": a.asn_org, "attempts": a.attempts}))
        .collect();

    let params: [&(dyn ToSql + Sync); 9] = [
        &onset,
        &round_to(spike.z, 2),
        &round_to(ratio, 3),
        &i32::try_from(asns.len())?,
        &round_to(spike.rate, 2),
        &round_to(spike.mean, 2),
        &json!(top_asns),
        &json!(pairs),
        &pattern,
    ];
    db.execute(
        "INSERT INTO campaign_events
             (onset_time, z_score, spike_ratio, new_asn_count,
              peak_rate_per_hour, baseline_rate_per_hour,
              new_asns, top_pairs, credential_pattern)
         VALUES ($1, $2::float8, $3::float8, $4, $5::float8, $6::float8, $7, $8, $9)
         ON CONFLICT (onset_time) DO UPDATE SET
             z_score            = EXCLUDED.z_score,
             spike_ratio        = EXCLUDED.spike_ratio,
             new_asn_count      = EXCLUDED.new_asn_count,
             peak_rate_per_hour = EXCLUDED.peak_rate_per_hour,
             new_asns           = EXCLUDED.new_asns,
             top_pairs          = EXCLUDED.top_pairs,
             credential_pattern = EXCLUDED.credential_pattern",
        &params,
    )?;
    Ok(())
}
