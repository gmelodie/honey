//! Caches country and ASN data from `MaxMind` or db-ip.com databases in `ip_geo_cache`.

use std::collections::HashMap;
use std::net::IpAddr;

use anyhow::Result;
use maxminddb::Reader;
use postgres::Client;

pub struct GeoReaders {
    country: Option<Reader<Vec<u8>>>,
    asn: Option<Reader<Vec<u8>>>,
}

fn open_reader(var: &str) -> Option<Reader<Vec<u8>>> {
    let path = std::env::var(var).ok()?;
    Reader::open_readfile(&path)
        .inspect_err(|e| eprintln!("WARNING: GeoIP database {path} failed to load: {e}"))
        .ok()
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CountryField {
    /// `MaxMind` `GeoLite2`: `{"country": {"iso_code": "US", "names": {"en": ...}}}`.
    Nested {
        iso_code: Option<String>,
        names: Option<HashMap<String, String>>,
    },
    /// db-ip.com lite: `{"country": "US"}`.
    Flat(String),
}

#[derive(serde::Deserialize)]
struct CountryRecord {
    country: Option<CountryField>,
}

#[derive(serde::Deserialize)]
struct AsnRecord {
    autonomous_system_number: Option<u32>,
    autonomous_system_organization: Option<String>,
}

#[derive(Default)]
struct Geo {
    country_iso: Option<String>,
    country_name: Option<String>,
    asn: Option<i32>,
    asn_org: Option<String>,
}

fn lookup_country(r: &Reader<Vec<u8>>, ip: IpAddr) -> Result<(Option<String>, Option<String>)> {
    let rec: Option<CountryRecord> = r.lookup(ip)?.decode()?;
    Ok(match rec.and_then(|r| r.country) {
        Some(CountryField::Nested { iso_code, names }) => {
            (iso_code, names.and_then(|mut n| n.remove("en")))
        }
        Some(CountryField::Flat(iso)) if !iso.is_empty() => (Some(iso), None),
        _ => (None, None),
    })
}

fn lookup_asn(r: &Reader<Vec<u8>>, ip: IpAddr) -> Result<(Option<i32>, Option<String>)> {
    let rec: Option<AsnRecord> = r.lookup(ip)?.decode()?;
    Ok(rec.map_or((None, None), |r| {
        (
            r.autonomous_system_number
                .and_then(|n| i32::try_from(n).ok()),
            r.autonomous_system_organization,
        )
    }))
}

impl GeoReaders {
    pub fn open() -> Self {
        Self {
            country: open_reader("GEOIP_COUNTRY_DB"),
            asn: open_reader("GEOIP_ASN_DB"),
        }
    }

    fn lookup(&self, ip: &str) -> Geo {
        let mut g = Geo::default();
        let Ok(addr) = ip.parse::<IpAddr>() else {
            eprintln!("  WARNING: not an IP address: {ip}");
            return g;
        };
        if let Some(r) = &self.country {
            match lookup_country(r, addr) {
                Ok((iso, name)) => (g.country_iso, g.country_name) = (iso, name),
                Err(e) => eprintln!("  WARNING: country lookup failed for {ip}: {e}"),
            }
        }
        if let Some(r) = &self.asn {
            match lookup_asn(r, addr) {
                Ok((asn, org)) => (g.asn, g.asn_org) = (asn, org),
                Err(e) => eprintln!("  WARNING: ASN lookup failed for {ip}: {e}"),
            }
        }
        g
    }
}

pub fn enrich_new_ips(db: &mut Client, readers: &GeoReaders) -> Result<()> {
    if readers.country.is_none() && readers.asn.is_none() {
        eprintln!("  GeoIP readers unavailable, skipping enrichment");
        return Ok(());
    }

    let ips: Vec<String> = db
        .query(
            "SELECT DISTINCT s.ip FROM sessions s
             WHERE s.ip IS NOT NULL
               AND NOT EXISTS (SELECT 1 FROM ip_geo_cache c
                               WHERE c.ip = s.ip AND c.country_iso IS NOT NULL)
             LIMIT 5000",
            &[],
        )?
        .iter()
        .map(|r| r.get(0))
        .collect();
    if ips.is_empty() {
        return Ok(());
    }

    let geo: Vec<Geo> = ips.iter().map(|ip| readers.lookup(ip)).collect();
    let col = |f: fn(&Geo) -> Option<String>| geo.iter().map(f).collect::<Vec<_>>();
    let iso = col(|g| g.country_iso.clone());
    let name = col(|g| g.country_name.clone());
    let org = col(|g| g.asn_org.clone());
    let asn: Vec<Option<i32>> = geo.iter().map(|g| g.asn).collect();

    db.execute(
        "INSERT INTO ip_geo_cache (ip, country_iso, country_name, asn, asn_org)
         SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::int4[], $5::text[])
         ON CONFLICT (ip) DO UPDATE SET
             country_iso  = EXCLUDED.country_iso,
             country_name = EXCLUDED.country_name,
             asn          = COALESCE(EXCLUDED.asn, ip_geo_cache.asn),
             asn_org      = COALESCE(EXCLUDED.asn_org, ip_geo_cache.asn_org),
             looked_up_at = NOW()
         WHERE ip_geo_cache.country_iso IS NULL",
        &[&ips, &iso, &name, &asn, &org],
    )?;
    println!("  Geo-enriched {} new IPs", ips.len());
    Ok(())
}
