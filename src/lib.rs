//! Layout engine: proportion plans, guillotine packing, layout search, SVG — compiled to WASM.
//!
//! **WASM API:** [`solve_layout_json`](wasm_api::solve_layout_json) only (JSON in/out).

mod guillotine;
mod layout_search;
mod proportion;
mod render_svg;
mod wasm_api;

pub use wasm_api::solve_layout_json;

// Native / test-only re-exports (not wasm-bindgen types).
pub use guillotine::{
    OrderSeed, PackInstance, PackOptions, PackResult, PlacedRect, SkuRow, expand_instances,
    pack_multi_stage, total_placed_area, trivial_area_feasible, verify_two_stage,
    verify_two_stage_hv, verify_two_stage_vh,
};
pub use layout_search::{LayoutSolution, ProductSpec, SolveOptions, solve_layout};
pub use proportion::{
    CountPlan, ProportionPattern, compute_proportion_pattern, counts_for_k, enumerate_count_plans,
    gcd, gcd_many, pages_for_k, suggest_k_max_area, total_overproduction,
};
pub use render_svg::{SheetLocale, SvgOptions, format_sheet_preview_title, render_layout_svg};
