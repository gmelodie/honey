//! Builds the reference Bloom filter from `BLOOM_WORDLIST_URLS`, one download at a time.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use honey::bloom::ScalableBloom;
use honey::{env_or, ts, write_atomic};

const ERROR_RATE: f64 = 0.001;

fn is_current_format(path: &Path) -> bool {
    File::open(path).is_ok_and(|f| ScalableBloom::read_from(&mut BufReader::new(f)).is_ok())
}

fn add_wordlist(
    http: &reqwest::blocking::Client,
    bf: &mut ScalableBloom,
    url: &str,
    total: &mut u64,
) -> Result<()> {
    let resp = http.get(url).send()?.error_for_status()?;
    for line in BufReader::new(resp).split(b'\n') {
        let line = line?;
        let word = String::from_utf8_lossy(&line);
        if word.is_empty() {
            continue;
        }
        bf.insert(&word);
        *total += 1;
        if *total % 100_000 == 0 {
            println!("[{}]   {total} items so far ...", ts());
        }
    }
    Ok(())
}

fn run(path: &Path, urls: &[String]) -> Result<()> {
    let http = reqwest::blocking::Client::builder().timeout(None).build()?;
    let mut bf = ScalableBloom::new(ERROR_RATE);
    let mut total = 0;
    for (i, url) in urls.iter().enumerate() {
        println!(
            "[{}] ({}/{}) Downloading {url} ...",
            ts(),
            i + 1,
            urls.len()
        );
        add_wordlist(&http, &mut bf, url, &mut total).with_context(|| format!("download {url}"))?;
        println!("[{}]   running total: {total} items", ts());
    }

    let mut data = Vec::new();
    bf.write_to(&mut data)?;
    write_atomic(path, &data)?;
    let size_mb = std::fs::metadata(path)?.len() as f64 / (1024.0 * 1024.0);
    println!("[{}] Done.", ts());
    println!("      Items processed:     {total}");
    println!("      Unique items:        {}", bf.len());
    println!("      False positive rate: {:.3}%", ERROR_RATE * 100.0);
    println!(
        "      Filter file size:    {size_mb:.1} MB  ({})",
        path.display()
    );
    Ok(())
}

fn main() -> ExitCode {
    let path = PathBuf::from(env_or("BLOOM_DIR", "/bloom")).join("reference.bloom");
    if is_current_format(&path) {
        println!(
            "[{}] Filter already exists at {}, skipping.",
            ts(),
            path.display()
        );
        return ExitCode::SUCCESS;
    }

    let urls: Vec<String> = env_or("BLOOM_WORDLIST_URLS", "")
        .split_whitespace()
        .map(String::from)
        .collect();
    if urls.is_empty() {
        eprintln!("[{}] BLOOM_WORDLIST_URLS is empty, nothing to build.", ts());
        return ExitCode::FAILURE;
    }

    match run(&path, &urls) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[{}] ERROR: {e:#}", ts());
            ExitCode::FAILURE
        }
    }
}
