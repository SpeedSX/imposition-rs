//! Port of `Layout/src/proportion.ts` — gcd-scaled ratios and count-plan enumeration.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProportionError {
    EmptyTargets,
    InvalidTarget,
    InvalidK,
}

impl std::fmt::Display for ProportionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProportionError::EmptyTargets => write!(f, "targets must be non-empty"),
            ProportionError::InvalidTarget => write!(f, "each target must be a positive integer"),
            ProportionError::InvalidK => write!(f, "k must be a positive integer"),
        }
    }
}

impl std::error::Error for ProportionError {}

/// Greatest common divisor of two non-negative integers (both zero => 0).
pub fn gcd(a: i32, b: i32) -> i32 {
    let mut x = a.abs();
    let mut y = b.abs();
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    x
}

/// GCD of a non-empty slice of non-negative integers.
pub fn gcd_many(values: &[i32]) -> i32 {
    if values.is_empty() {
        return 0;
    }
    values.iter().fold(values[0], |acc, &v| gcd(acc, v))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProportionPattern {
    /// d = gcd(T_1,…,T_n)
    pub d: i32,
    /// p_i = T_i / d
    pub p: Vec<i32>,
}

pub fn compute_proportion_pattern(targets: &[i32]) -> Result<ProportionPattern, ProportionError> {
    if targets.is_empty() {
        return Err(ProportionError::EmptyTargets);
    }
    for &t in targets {
        if t <= 0 {
            return Err(ProportionError::InvalidTarget);
        }
    }
    let d = gcd_many(targets);
    let p: Vec<i32> = targets.iter().map(|&t| t / d).collect();
    Ok(ProportionPattern { d, p })
}

/// Per-page counts c_i = k * p_i
pub fn counts_for_k(pattern: &ProportionPattern, k: i32) -> Result<Vec<i32>, ProportionError> {
    if k < 1 {
        return Err(ProportionError::InvalidK);
    }
    Ok(pattern.p.iter().map(|&pi| k * pi).collect())
}

fn ceil_div_pos(numer: i32, denom: i32) -> i32 {
    debug_assert!(numer > 0 && denom > 0);
    (numer + denom - 1) / denom
}

/// P(k) = max_i ceil(T_i / (k * p_i))
pub fn pages_for_k(targets: &[i32], pattern: &ProportionPattern, k: i32) -> Result<i32, ProportionError> {
    let c = counts_for_k(pattern, k)?;
    let mut max_p = 0;
    for i in 0..targets.len() {
        let pages = ceil_div_pos(targets[i], c[i]);
        if pages > max_p {
            max_p = pages;
        }
    }
    Ok(max_p)
}

pub fn total_overproduction(targets: &[i32], counts_per_page: &[i32], pages: i32) -> i32 {
    let mut sum = 0;
    for i in 0..targets.len() {
        sum += counts_per_page[i] * pages - targets[i];
    }
    sum
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountPlan {
    pub counts: Vec<i32>,
    pub pages: i32,
    /// If this plan is exactly `k * primitive p`, that k; otherwise None.
    pub gcd_k: Option<i32>,
}

fn infer_gcd_k(counts: &[i32], pattern: &ProportionPattern) -> Option<i32> {
    if pattern.p.is_empty() || counts.is_empty() {
        return None;
    }
    if pattern.p[0] == 0 {
        return None;
    }
    let k0 = counts[0] / pattern.p[0];
    if counts[0] % pattern.p[0] != 0 || k0 < 1 {
        return None;
    }
    for (i, item) in counts.iter().enumerate() {
        if *item != k0 * pattern.p[i] {
            return None;
        }
    }
    Some(k0)
}

/// All distinct per-sheet count vectors to try (gcd-scaled primitive ratios plus ceil families).
pub fn enumerate_count_plans(targets: &[i32], pattern: &ProportionPattern, k_max: i32) -> Vec<CountPlan> {
    use std::collections::HashMap;

    let mut map: HashMap<String, CountPlan> = HashMap::new();

    let upsert = |map: &mut HashMap<String, CountPlan>, counts: Vec<i32>, gcd_k: Option<i32>| {
        if counts.iter().any(|&c| c < 1) {
            return;
        }
        let pages = targets
            .iter()
            .zip(counts.iter())
            .map(|(&t, &c)| ceil_div_pos(t, c))
            .max()
            .unwrap_or(0);
        let key = counts.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        let merged_gcd_k = match map.get(&key) {
            Some(prev) => gcd_k.or(prev.gcd_k),
            None => gcd_k,
        };
        map.insert(
            key,
            CountPlan {
                counts,
                pages,
                gcd_k: merged_gcd_k,
            },
        );
    };

    for k in 1..=k_max {
        if let Ok(c) = counts_for_k(pattern, k) {
            upsert(&mut map, c, Some(k));
        }
    }

    let max_t = *targets.iter().max().unwrap_or(&1);
    for p_try in 1..=max_t {
        let counts: Vec<i32> = targets.iter().map(|&t| ceil_div_pos(t, p_try)).collect();
        let inferred = infer_gcd_k(&counts, pattern);
        upsert(&mut map, counts, inferred);
    }

    map.into_values().collect()
}

/// Upper bound on k from trivial area (optional coarse cap).
pub fn suggest_k_max_area(
    pattern: &ProportionPattern,
    sheet_w: f64,
    sheet_h: f64,
    product_dims: &[(f64, f64)],
) -> i32 {
    let sheet_area = sheet_w * sheet_h;
    let mut cap = f64::INFINITY;
    for i in 0..pattern.p.len() {
        let Some(&(w, h)) = product_dims.get(i) else {
            continue;
        };
        let a = w * h;
        if a <= 0.0 {
            continue;
        }
        let per_k = (pattern.p[i] as f64) * a;
        if per_k <= 0.0 {
            continue;
        }
        let ki = (sheet_area / per_k).floor();
        cap = cap.min(ki);
    }
    if cap.is_infinite() {
        1
    } else {
        (cap as i32).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s1_01_motivating_ratio() {
        let t = [1000, 2000];
        let pat = compute_proportion_pattern(&t).unwrap();
        assert_eq!(pat.d, 1000);
        assert_eq!(pat.p, vec![1, 2]);
        assert_eq!(counts_for_k(&pat, 1).unwrap(), vec![1, 2]);
        assert_eq!(pages_for_k(&t, &pat, 1).unwrap(), 1000);
        assert_eq!(counts_for_k(&pat, 2).unwrap(), vec![2, 4]);
        assert_eq!(pages_for_k(&t, &pat, 2).unwrap(), 500);
        assert_eq!(counts_for_k(&pat, 1000).unwrap(), vec![1000, 2000]);
        assert_eq!(pages_for_k(&t, &pat, 1000).unwrap(), 1);
    }

    #[test]
    fn s1_02_t_6_10_15() {
        let t = [6, 10, 15];
        let pat = compute_proportion_pattern(&t).unwrap();
        assert_eq!(pat.d, 1);
        assert_eq!(pat.p, vec![6, 10, 15]);
        assert_eq!(pages_for_k(&t, &pat, 1).unwrap(), 1);
    }

    #[test]
    fn s1_03_equal_triple() {
        let t = [100, 100, 100];
        let pat = compute_proportion_pattern(&t).unwrap();
        assert_eq!(pat.d, 100);
        assert_eq!(pat.p, vec![1, 1, 1]);
        assert_eq!(counts_for_k(&pat, 1).unwrap(), vec![1, 1, 1]);
        assert_eq!(pages_for_k(&t, &pat, 1).unwrap(), 100);
    }

    #[test]
    fn s1_04_primitive_ratio_12_18() {
        let t = [12, 18];
        let pat = compute_proportion_pattern(&t).unwrap();
        assert_eq!(pat.d, 6);
        assert_eq!(pat.p, vec![2, 3]);
        assert_eq!(pages_for_k(&t, &pat, 1).unwrap(), 6);
    }

    #[test]
    fn s1_05_triple_gcd() {
        let t = [1000, 2000, 3000];
        let pat = compute_proportion_pattern(&t).unwrap();
        assert_eq!(pat.d, 1000);
        assert_eq!(pat.p, vec![1, 2, 3]);
        assert_eq!(counts_for_k(&pat, 2).unwrap(), vec![2, 4, 6]);
        assert_eq!(pages_for_k(&t, &pat, 2).unwrap(), 500);
    }

    #[test]
    fn s1_06_monotonicity_of_p_k() {
        let t = [1000, 2000];
        let pat = compute_proportion_pattern(&t).unwrap();
        let mut prev = i32::MAX;
        for k in 1..=50 {
            let p = pages_for_k(&t, &pat, k).unwrap();
            let manual: i32 = t
                .iter()
                .zip(pat.p.iter())
                .map(|(&ti, &pi)| ceil_div_pos(ti, k * pi))
                .max()
                .unwrap();
            assert_eq!(p, manual);
            assert!(p <= prev);
            prev = p;
        }
    }

    #[test]
    fn gcd_many_pair() {
        assert_eq!(gcd_many(&[1000, 2000]), 1000);
    }
}
