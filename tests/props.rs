use honey::bloom::ScalableBloom;
use honey::cowrie::Batch;
use honey::rules::detect_suffix;
use honey::trends::{TrendRow, classify};
use proptest::prelude::*;
use serde_json::{Value, json};

const EVENTS: [&str; 10] = [
    "cowrie.session.connect",
    "cowrie.session.closed",
    "cowrie.client.version",
    "cowrie.client.size",
    "cowrie.login.success",
    "cowrie.login.failed",
    "cowrie.command.input",
    "cowrie.log.closed",
    "cowrie.session.file_download",
    "cowrie.client.fingerprint",
];

fn scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::from),
        any::<i64>().prop_map(Value::from),
        "\\PC{0,12}".prop_map(Value::from),
        "[a\u{0}]{0,4}".prop_map(Value::from),
    ]
}

fn event() -> impl Strategy<Value = Value> {
    let keys = [
        "src_ip",
        "timestamp",
        "sensor",
        "protocol",
        "version",
        "width",
        "height",
        "username",
        "password",
        "input",
        "realm",
        "success",
        "ttylog",
        "filename",
        "url",
        "outfile",
        "shasum",
        "fingerprint",
    ];
    (
        prop::sample::select(EVENTS.to_vec()),
        prop::collection::vec((prop::sample::select(keys.to_vec()), scalar()), 0..8),
        prop::sample::select(vec!["s1", "s2", ""]),
    )
        .prop_map(|(eid, fields, sid)| {
            let mut ev = json!({"eventid": eid, "session": sid});
            for (k, v) in fields {
                ev[k] = v;
            }
            ev
        })
}

proptest! {
    #[test]
    fn bloom_has_no_false_negatives(items in prop::collection::vec("\\PC{0,16}", 0..300)) {
        let mut bf = ScalableBloom::new(0.001);
        for it in &items {
            bf.insert(it);
        }
        let mut bytes = Vec::new();
        bf.write_to(&mut bytes).unwrap();
        let back = ScalableBloom::read_from(&mut bytes.as_slice()).unwrap();
        for it in &items {
            prop_assert!(bf.contains(it));
            prop_assert!(back.contains(it));
        }
    }

    #[test]
    fn bloom_reader_rejects_garbage_without_panic(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
        let _ = ScalableBloom::read_from(&mut bytes.as_slice());
    }

    #[test]
    fn detected_suffix_is_a_strict_suffix(pw in "\\PC{0,12}(20[12][0-9])?(!|@|#|1|12|123|!@#)?") {
        if let Some(sfx) = detect_suffix(&pw) {
            prop_assert!(pw.ends_with(&sfx));
            prop_assert!(pw.len() > sfx.len());
        }
    }

    #[test]
    fn trends_are_disjoint_and_pass_the_floor(rows in prop::collection::vec((0i64..50, 0i64..50), 0..40)) {
        let rows: Vec<TrendRow> = rows
            .iter()
            .enumerate()
            .map(|(i, &(recent, baseline))| TrendRow { password: i.to_string(), recent, baseline })
            .collect();
        let by_name: Vec<(i64, i64)> = rows.iter().map(|r| (r.recent, r.baseline)).collect();
        let t = classify(rows);
        for pw in &t.trending {
            let (r, b) = by_name[pw.parse::<usize>().unwrap()];
            prop_assert!(r >= t.min_count && r > b);
            prop_assert!(!t.dying.contains(pw));
        }
        for pw in &t.dying {
            let (r, b) = by_name[pw.parse::<usize>().unwrap()];
            prop_assert!(b >= t.min_count && b > r);
        }
    }

    #[test]
    fn cowrie_collect_never_panics_and_strips_nul(events in prop::collection::vec(event(), 0..30)) {
        let n = events.len();
        let b = Batch::collect(events);
        prop_assert!(b.sessions.len() <= 2);
        prop_assert!(b.auth.len() + b.input.len() + b.downloads.len() <= n);
        for a in &b.auth {
            prop_assert!(!a.username.as_deref().unwrap_or("").contains('\0'));
            prop_assert!(!a.password.as_deref().unwrap_or("").contains('\0'));
        }
        for s in &b.sessions {
            prop_assert!(s.termsize.chars().count() <= 7, "termsize column is VARCHAR(7): {}", s.termsize);
        }
    }
}
