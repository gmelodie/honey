//! Hashcat and John mutation rules derived from observed password suffixes and case.

use std::collections::HashMap;

/// Longest first, so "123" wins over "1" and "3".
const SPECIALS: [&str; 13] = [
    "!@#", "123", "12", "01", "1!", "2!", "3!", "!", "@", "#", "$", "1", "2",
];

const CASE_SAMPLE: usize = 500;
const CASE_THRESHOLD: f64 = 0.05;
const TOP_SUFFIXES: usize = 50;

fn years() -> impl Iterator<Item = String> {
    (2018..2030).map(|y| y.to_string())
}

pub fn detect_suffix(pw: &str) -> Option<String> {
    let strict_suffix = |s: &str| pw.ends_with(s) && pw.len() > s.len();

    for year in years() {
        if let Some(sfx) = SPECIALS
            .iter()
            .find(|s| strict_suffix(&format!("{year}{s}")))
        {
            return Some(format!("{year}{sfx}"));
        }
        if strict_suffix(&year) {
            return Some(year);
        }
    }
    SPECIALS
        .iter()
        .find(|s| strict_suffix(s))
        .map(std::string::ToString::to_string)
}

/// Case rules (`l`, `u`, `c`) that cover at least `CASE_THRESHOLD` of the first `CASE_SAMPLE` passwords.
pub fn detect_case_rules(passwords: &[String]) -> Vec<&'static str> {
    let with_alpha: Vec<&String> = passwords
        .iter()
        .take(CASE_SAMPLE)
        .filter(|pw| pw.chars().any(char::is_alphabetic))
        .collect();
    if with_alpha.is_empty() {
        return Vec::new();
    }

    let mut counts = [("l", 0usize), ("c", 0), ("u", 0)];
    for pw in &with_alpha {
        let alpha: Vec<char> = pw.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha.iter().all(|c| c.is_lowercase()) {
            counts[0].1 += 1;
        } else if alpha.iter().all(|c| c.is_uppercase()) {
            counts[2].1 += 1;
        } else if pw.chars().next().is_some_and(char::is_uppercase)
            && pw
                .chars()
                .skip(1)
                .filter(|c| c.is_alphabetic())
                .all(char::is_lowercase)
        {
            counts[1].1 += 1;
        }
    }

    counts.sort_by_key(|c| std::cmp::Reverse(c.1));
    let total = with_alpha.len() as f64;
    counts
        .iter()
        .filter(|(_, n)| *n as f64 / total >= CASE_THRESHOLD)
        .map(|(rule, _)| *rule)
        .collect()
}

/// Suffixes ranked by count; ties keep first-seen order.
fn ranked_suffixes(passwords: &[String]) -> Vec<String> {
    let mut order: Vec<(String, usize)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for sfx in passwords.iter().filter_map(|pw| detect_suffix(pw)) {
        if let Some(&i) = index.get(&sfx) {
            order[i].1 += 1;
        } else {
            index.insert(sfx.clone(), order.len());
            order.push((sfx, 1));
        }
    }

    order.sort_by_key(|o| std::cmp::Reverse(o.1));
    order
        .into_iter()
        .take(TOP_SUFFIXES)
        .map(|(s, _)| s)
        .collect()
}

fn hashcat_append(suffix: &str) -> String {
    suffix
        .chars()
        .map(|c| format!("${c}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns `(hashcat.rule, john.rule)` file contents.
pub fn generate_rules(passwords: &[String]) -> (String, String) {
    let suffixes = ranked_suffixes(passwords);
    let case_rules = detect_case_rules(passwords);

    let mut hc = vec![
        "# autopot — derived mutation rules (hashcat)".to_string(),
        "# usage: hashcat -a 0 -r hashcat.rule hashes.txt wordlist.txt".to_string(),
    ];
    let mut jt = vec![
        "[List.Rules:autopot]".to_string(),
        "# autopot — derived mutation rules".to_string(),
        "# append section to john.conf, then:".to_string(),
        "# john --wordlist=words.txt --rules=autopot hashes".to_string(),
    ];

    if !case_rules.is_empty() {
        hc.push(String::new());
        jt.push(String::new());
        hc.extend(case_rules.iter().map(std::string::ToString::to_string));
        jt.extend(case_rules.iter().map(std::string::ToString::to_string));
    }

    if !suffixes.is_empty() {
        hc.push(String::new());
        jt.push(String::new());
    }

    for sfx in &suffixes {
        let h = hashcat_append(sfx);
        let j = format!("Az\"{sfx}\"");
        hc.extend([h.clone(), format!("c {h}")]);
        jt.extend([j.clone(), format!("c{j}")]);
    }

    (hc.join("\n") + "\n", jt.join("\n") + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(std::string::ToString::to_string).collect()
    }

    #[test]
    fn suffix_table() {
        let cases: &[(&str, Option<&str>)] = &[
            ("", None),
            ("2024", None),
            ("!", None),
            ("123", None),
            ("12", Some("2")),
            ("!@#", Some("#")),
            ("a!", Some("!")),
            ("a1", Some("1")),
            ("a2", Some("2")),
            ("a3", None),
            ("a12", Some("12")),
            ("a123", Some("123")),
            ("a1231", Some("1")),
            ("a01", Some("01")),
            ("a!@#", Some("!@#")),
            ("a3!", Some("3!")),
            ("a2024", Some("2024")),
            ("a2018", Some("2018")),
            ("a2029", Some("2029")),
            ("a2017", None),
            ("a2030", None),
            ("2024!", Some("!")),
            ("a2024!", Some("2024!")),
            ("a2024!@#", Some("2024!@#")),
            ("a2024123", Some("2024123")),
            ("a202412", Some("202412")),
            ("a20241", Some("20241")),
            ("a2024$", Some("2024$")),
            ("a20182019", Some("2019")),
            ("2024a2024", Some("2024")),
            ("héllo2024", Some("2024")),
            ("pass word 2024\n", None),
        ];
        for (pw, want) in cases {
            assert_eq!(detect_suffix(pw).as_deref(), *want, "{pw:?}");
        }
    }

    #[test]
    fn case_rules_empty_and_no_alpha() {
        assert!(detect_case_rules(&[]).is_empty());
        assert!(detect_case_rules(&v(&["123", "!!", ""])).is_empty());
    }

    #[test]
    fn case_rules_classify_each_shape() {
        assert_eq!(detect_case_rules(&v(&["abc1"])), ["l"]);
        assert_eq!(detect_case_rules(&v(&["ABC1"])), ["u"]);
        assert_eq!(detect_case_rules(&v(&["Abc1"])), ["c"]);
        assert_eq!(detect_case_rules(&v(&["A"])), ["u"]);
        assert!(detect_case_rules(&v(&["aBc"])).is_empty());
        assert!(detect_case_rules(&v(&["1Abc"])).is_empty());
    }

    fn mix(l: usize, c: usize, u: usize) -> Vec<String> {
        let mut p = vec!["abc".to_string(); l];
        p.extend(vec!["Abc".to_string(); c]);
        p.extend(vec!["ABC".to_string(); u]);
        p
    }

    #[test]
    fn case_rules_threshold_is_five_percent_inclusive() {
        assert_eq!(detect_case_rules(&mix(95, 5, 0)), ["l", "c"]);
        assert_eq!(detect_case_rules(&mix(96, 4, 0)), ["l"]);
        assert_eq!(detect_case_rules(&mix(95, 0, 5)), ["l", "u"]);
    }

    #[test]
    fn case_rules_order_by_count_with_lcu_tie_order() {
        assert_eq!(detect_case_rules(&mix(10, 10, 10)), ["l", "c", "u"]);
        assert_eq!(detect_case_rules(&mix(2, 5, 20)), ["u", "c", "l"]);
    }

    #[test]
    fn case_rules_only_sample_first_500_and_ignore_non_alpha_in_denominator() {
        let mut p = vec!["abc".to_string(); 500];
        p.extend(vec!["ABC".to_string(); 500]);
        assert_eq!(detect_case_rules(&p), ["l"]);
        let mut q = vec!["123".to_string(); 1000];
        q.push("ABC".into());
        assert!(detect_case_rules(&q[..500]).is_empty());
        assert_eq!(detect_case_rules(&q[999..]), ["u"]);
    }

    #[test]
    fn case_rules_501st_password_is_outside_the_sample() {
        let mut p = mix(475, 0, 25);
        p.push("abc".into());
        assert_eq!(detect_case_rules(&p), ["l", "u"]);
    }

    #[test]
    fn generate_rules_empty_input_has_headers_only() {
        let (hc, jt) = generate_rules(&[]);
        assert_eq!(
            hc,
            "# autopot \u{2014} derived mutation rules (hashcat)\n# usage: hashcat -a 0 -r hashcat.rule hashes.txt wordlist.txt\n"
        );
        assert_eq!(
            jt,
            "[List.Rules:autopot]\n# autopot \u{2014} derived mutation rules\n# append section to john.conf, then:\n# john --wordlist=words.txt --rules=autopot hashes\n"
        );
    }

    #[test]
    fn generate_rules_exact_output_with_case_and_tied_suffixes_in_first_seen_order() {
        let (hc, jt) = generate_rules(&v(&["x2", "y1", "z1", "w2", "e!", "plain"]));
        let hc_body = hc.split_once("\n\n").unwrap().1;
        assert_eq!(hc_body, "l\n\n$2\nc $2\n$1\nc $1\n$!\nc $!\n");
        let jt_body = jt.split_once("\n\n").unwrap().1;
        assert_eq!(
            jt_body,
            "l\n\nAz\"2\"\ncAz\"2\"\nAz\"1\"\ncAz\"1\"\nAz\"!\"\ncAz\"!\"\n"
        );
    }

    #[test]
    fn generate_rules_higher_count_beats_first_seen() {
        let (hc, _) = generate_rules(&v(&["a2", "b1", "c1"]));
        assert!(hc.ends_with("$1\nc $1\n$2\nc $2\n"), "{hc}");
    }

    #[test]
    fn generate_rules_multichar_suffix_is_space_separated_per_char() {
        let (hc, jt) = generate_rules(&v(&["Ab2024!"]));
        assert!(
            hc.ends_with("\n\nc\n\n$2 $0 $2 $4 $!\nc $2 $0 $2 $4 $!\n"),
            "{hc}"
        );
        assert!(jt.ends_with("Az\"2024!\"\ncAz\"2024!\"\n"), "{jt}");
    }

    #[test]
    fn generate_rules_without_case_rules_has_single_blank_before_suffixes() {
        let (hc, _) = generate_rules(&v(&["1!2"]));
        assert!(hc.ends_with("wordlist.txt\n\n$2\nc $2\n"), "{hc}");
        let (hc, _) = generate_rules(&v(&["12345"]));
        assert!(hc.ends_with("wordlist.txt\n"), "{hc}");
    }

    #[test]
    fn generate_rules_keeps_only_top_50_suffixes() {
        let mut pws = Vec::new();
        for y in 2018..2030 {
            for s in SPECIALS {
                pws.push(format!("a{y}{s}"));
            }
        }
        let (hc, jt) = generate_rules(&pws);
        assert_eq!(hc.lines().filter(|l| l.starts_with("c $")).count(), 50);
        assert_eq!(jt.lines().filter(|l| l.starts_with("cAz")).count(), 50);
        assert!(hc.contains("$2 $0 $1 $8 $!\n"));
        assert!(!hc.contains("$2 $0 $2 $9 $2\n"));
    }
}
