//! Split passwords into trending and dying sets by comparing a recent window to its baseline.

pub struct TrendRow {
    pub password: String,
    pub recent: i64,
    pub baseline: i64,
}

pub struct Trends {
    pub trending: Vec<String>,
    pub dying: Vec<String>,
    pub min_count: i64,
}

/// Poisson noise floor: std dev of random arrivals is about sqrt(mean), never below 2.
pub fn noise_floor(rows: &[TrendRow]) -> i64 {
    if rows.is_empty() {
        return 2;
    }
    let total: i64 = rows.iter().map(|r| r.recent + r.baseline).sum();
    let mean = total as f64 / rows.len() as f64;
    #[expect(clippy::cast_possible_truncation, reason = "sqrt of a row count")]
    let floor = mean.sqrt().round_ties_even() as i64;
    floor.max(2)
}

/// Compares each password's share of its period, so a change in total volume moves no password.
pub fn classify(rows: Vec<TrendRow>) -> Trends {
    let min_count = noise_floor(&rows);
    let recent_total = i128::from(rows.iter().map(|r| r.recent).sum::<i64>());
    let baseline_total = i128::from(rows.iter().map(|r| r.baseline).sum::<i64>());
    let recent_scaled = |r: &TrendRow| i128::from(r.recent) * baseline_total;
    let baseline_scaled = |r: &TrendRow| i128::from(r.baseline) * recent_total;

    let (mut trending, mut dying) = (Vec::new(), Vec::new());
    for r in rows {
        if r.recent >= min_count && recent_scaled(&r) > baseline_scaled(&r) {
            trending.push(r);
        } else if r.baseline >= min_count && baseline_scaled(&r) > recent_scaled(&r) {
            dying.push(r);
        }
    }

    let growth = |r: &TrendRow| r.recent as f64 / (r.baseline as f64).max(0.1);
    trending.sort_by(|a, b| growth(b).total_cmp(&growth(a)));
    dying.sort_by_key(|r| std::cmp::Reverse(baseline_scaled(r) - recent_scaled(r)));

    Trends {
        trending: trending.into_iter().map(|r| r.password).collect(),
        dying: dying.into_iter().map(|r| r.password).collect(),
        min_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(p: &str, recent: i64, baseline: i64) -> TrendRow {
        TrendRow {
            password: p.into(),
            recent,
            baseline,
        }
    }

    #[test]
    fn empty_rows_floor_is_two() {
        assert_eq!(noise_floor(&[]), 2);
        let t = classify(vec![]);
        assert_eq!((t.min_count, t.trending.len(), t.dying.len()), (2, 0, 0));
    }

    #[test]
    fn floor_rounds_half_to_even_like_python() {
        // mean 6.25 -> sqrt 2.5 -> 2 (half-up would give 3)
        assert_eq!(
            noise_floor(&[
                row("a", 6, 0),
                row("b", 6, 0),
                row("c", 6, 0),
                row("d", 7, 0)
            ]),
            2
        );
        // mean 20.25 -> sqrt 4.5 -> 4 (half-up would give 5)
        assert_eq!(
            noise_floor(&[
                row("a", 20, 0),
                row("b", 20, 0),
                row("c", 20, 0),
                row("d", 21, 0)
            ]),
            4
        );
        // mean 12.25 -> sqrt 3.5 -> 4
        assert_eq!(
            noise_floor(&[
                row("a", 12, 0),
                row("b", 12, 0),
                row("c", 12, 0),
                row("d", 13, 0)
            ]),
            4
        );
    }

    #[test]
    fn floor_never_below_two() {
        assert_eq!(noise_floor(&[row("a", 0, 0)]), 2);
        assert_eq!(noise_floor(&[row("a", 1, 0)]), 2);
    }

    #[test]
    fn trending_sorted_by_growth_and_dying_by_decline_with_stable_ties() {
        let t = classify(vec![
            row("slow", 9, 5),
            row("new", 3, 0),
            row("tie1", 4, 2),
            row("tie2", 4, 2),
            row("d_small", 0, 3),
            row("d_big", 1, 9),
            row("d_tie", 0, 3),
            row("flat", 5, 5),
            row("noise", 1, 0),
            row("b1", 5, 1),
            row("d_mid", 4, 6),
        ]);
        assert_eq!(t.min_count, 3);
        assert_eq!(t.trending, ["new", "b1", "tie1", "tie2", "slow"]);
        assert_eq!(t.dying, ["d_big", "d_small", "d_tie", "d_mid"]);
    }

    #[test]
    fn traffic_growth_still_finds_dying_by_share() {
        let t = classify(vec![
            row("grew", 90, 10),
            row("shrank", 20, 10),
            row("held", 55, 10),
        ]);
        assert_eq!(t.trending, ["grew"]);
        assert_eq!(t.dying, ["shrank"]);
    }

    #[test]
    fn recent_equal_to_floor_counts_and_below_does_not() {
        let t = classify(vec![row("at", 2, 0), row("below", 1, 0), row("dyat", 0, 2)]);
        assert_eq!(t.min_count, 2);
        assert_eq!(t.trending, ["at"]);
        assert_eq!(t.dying, ["dyat"]);
    }
}
