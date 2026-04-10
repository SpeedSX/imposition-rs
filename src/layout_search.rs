//! Port of `Layout/src/layoutSearch.ts` — enumerate count plans + pack.

use crate::guillotine::{
    expand_instances, pack_multi_stage, trivial_area_feasible, OrderSeed, PackInstance, PackOptions,
    PackResult, SkuRow,
};
use crate::proportion::{
    compute_proportion_pattern, enumerate_count_plans, suggest_k_max_area, total_overproduction,
    ProportionPattern,
};

const PACK_SEEDS: &[OrderSeed] = &[
    OrderSeed::AreaDesc,
    OrderSeed::AreaAsc,
    OrderSeed::Default,
    OrderSeed::WidthDesc,
];

#[derive(Debug, Clone)]
pub struct ProductSpec {
    pub id: String,
    pub w: i32,
    pub h: i32,
    pub target: i32,
    pub allow_rotation: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LayoutSolution {
    pub k: i32,
    pub counts_per_page: Vec<i32>,
    pub pages: i32,
    pub overproduction: i32,
    pub utilization: f64,
    pub pack: PackResult,
    pub pattern: ProportionPattern,
}

pub struct SolveOptions {
    pub k_max: Option<i32>,
    pub allow_rotation: bool,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            k_max: None,
            allow_rotation: true,
        }
    }
}

fn targets_from_products(products: &[ProductSpec]) -> Vec<i32> {
    products.iter().map(|p| p.target).collect()
}

fn instances_from_counts(products: &[ProductSpec], counts: &[i32]) -> Vec<PackInstance> {
    let rows: Vec<SkuRow> = products
        .iter()
        .zip(counts.iter())
        .map(|(p, &c)| SkuRow {
            id: p.id.clone(),
            w: p.w,
            h: p.h,
            count: c,
            allow_rotation: p.allow_rotation,
        })
        .collect();
    expand_instances(&rows)
}

/// Minimize press sheets P, then maximize utilization, then minimize overproduction.
#[must_use]
pub fn solve_layout(
    sheet_w: i32,
    sheet_h: i32,
    products: &[ProductSpec],
    options: &SolveOptions,
) -> Option<LayoutSolution> {
    if products.is_empty() {
        return None;
    }
    let targets = targets_from_products(products);
    let pattern = compute_proportion_pattern(&targets).ok()?;
    let dims: Vec<(f64, f64)> = products.iter().map(|p| (f64::from(p.w), f64::from(p.h))).collect();
    let area_cap = suggest_k_max_area(&pattern, f64::from(sheet_w), f64::from(sheet_h), &dims);
    let k_cap = options.k_max.unwrap_or(500);
    let k_max = k_cap.min(area_cap.max(1) + 100).clamp(1, 2000);

    let plans = enumerate_count_plans(&targets, &pattern, k_max);
    let allow_rot = options.allow_rotation;

    let mut feasible: Vec<LayoutSolution> = Vec::new();

    for plan in plans {
        let instances = instances_from_counts(products, &plan.counts);
        if !trivial_area_feasible(&instances, sheet_w, sheet_h) {
            continue;
        }

        let packs = pack_multi_stage(
            sheet_w,
            sheet_h,
            &instances,
            &PackOptions {
                max_solutions: 16,
                seeds: PACK_SEEDS,
                allow_rotation: allow_rot,
            },
        );
        if packs.is_empty() {
            continue;
        }

        let best_pack = packs
            .iter()
            .max_by(|a, b| {
                a.utilization
                    .partial_cmp(&b.utilization)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()?;

        let over = total_overproduction(&targets, &plan.counts, plan.pages);
        let gcd_k = plan.gcd_k.unwrap_or(0);
        feasible.push(LayoutSolution {
            k: gcd_k,
            counts_per_page: plan.counts.clone(),
            pages: plan.pages,
            overproduction: over,
            utilization: best_pack.utilization,
            pack: best_pack,
            pattern: pattern.clone(),
        });
    }

    if feasible.is_empty() {
        return None;
    }

    let min_pages = feasible.iter().map(|s| s.pages).min()?;
    let mut tier: Vec<LayoutSolution> = feasible.into_iter().filter(|s| s.pages == min_pages).collect();
    tier.sort_by(|a, b| {
        b.utilization
            .partial_cmp(&a.utilization)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.overproduction.cmp(&b.overproduction))
    });
    tier.into_iter().next()
}
