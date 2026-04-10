//! Minimal WASM surface: JSON in → JSON out (layout + SVG).

use crate::layout_search::{solve_layout, ProductSpec, SolveOptions};
use crate::render_svg::{render_layout_svg, SvgOptions};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveLayoutRequest {
    pub sheet_w: i32,
    pub sheet_h: i32,
    pub products: Vec<ProductSpecWire>,
    #[serde(default)]
    pub k_max: Option<i32>,
    #[serde(default)]
    pub allow_rotation: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSpecWire {
    pub id: String,
    pub w: i32,
    pub h: i32,
    pub target: i32,
    #[serde(default)]
    pub allow_rotation: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveLayoutResponse {
    pub k: i32,
    pub counts_per_page: Vec<i32>,
    pub pages: i32,
    pub overproduction: i32,
    pub utilization: f64,
    pub pattern: PatternWire,
    pub pack: PackWire,
    pub svg: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternWire {
    pub d: i32,
    pub p: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackWire {
    pub mode: String,
    pub utilization: f64,
    pub rects: Vec<RectWire>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RectWire {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub product_id: String,
    pub rotated: bool,
}

fn build_title(
    sheet_w: i32,
    sheet_h: i32,
    sol: &crate::layout_search::LayoutSolution,
    products: &[ProductSpecWire],
    default_allow_rot: bool,
) -> String {
    let per_page: String = products
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{}×{}", p.id, sol.counts_per_page[i]))
        .collect::<Vec<_>>()
        .join(", ");
    let k_label = if sol.k > 0 {
        sol.k.to_string()
    } else {
        "— (mixed)".to_string()
    };
    let rot_note = format!(
        "sheet default {}",
        if default_allow_rot { "on" } else { "off" }
    );
    format!(
        "Wp×Hp {}×{} · k={} · {} · P={} sheets · util {:.1}% · mode {} · {}",
        sheet_w,
        sheet_h,
        k_label,
        per_page,
        sol.pages,
        sol.utilization * 100.0,
        sol.pack.mode.as_str(),
        rot_note
    )
}

fn wire_response(
    sheet_w: i32,
    sheet_h: i32,
    sol: crate::layout_search::LayoutSolution,
    products: &[ProductSpecWire],
    default_allow_rot: bool,
    stroke_width: f64,
) -> SolveLayoutResponse {
    let title = build_title(sheet_w, sheet_h, &sol, products, default_allow_rot);
    let svg = render_layout_svg(
        sheet_w,
        sheet_h,
        &sol.pack.rects,
        &SvgOptions {
            stroke_width,
            show_labels: true,
            title,
        },
    );

    let rects: Vec<RectWire> = sol
        .pack
        .rects
        .iter()
        .map(|r| RectWire {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
            product_id: r.product_id.clone(),
            rotated: r.rotated,
        })
        .collect();

    SolveLayoutResponse {
        k: sol.k,
        counts_per_page: sol.counts_per_page,
        pages: sol.pages,
        overproduction: sol.overproduction,
        utilization: sol.utilization,
        pattern: PatternWire {
            d: sol.pattern.d,
            p: sol.pattern.p,
        },
        pack: PackWire {
            mode: sol.pack.mode.as_str().to_string(),
            utilization: sol.pack.utilization,
            rects,
        },
        svg,
    }
}

/// Full layout solve + SVG preview. Input/output JSON uses camelCase (TypeScript-friendly).
///
/// # Errors
/// Throws a JS `Error` on JSON parse failure or when no feasible layout exists.
#[wasm_bindgen(js_name = solveLayoutJson)]
pub fn solve_layout_json(input: &str) -> Result<String, JsValue> {
    let req: SolveLayoutRequest = serde_json::from_str(input).map_err(|e| {
        JsValue::from(js_sys::Error::new(&format!("Invalid JSON: {e}")))
    })?;

    if req.sheet_w < 1 || req.sheet_h < 1 {
        return Err(JsValue::from(js_sys::Error::new(
            "sheet width and height must be positive integers",
        )));
    }

    if req.products.is_empty() {
        return Err(JsValue::from(js_sys::Error::new(
            "at least one product is required",
        )));
    }

    let mut seen_ids = std::collections::HashSet::<&str>::new();
    for p in &req.products {
        if p.w < 1 || p.h < 1 || p.target < 1 {
            return Err(JsValue::from(js_sys::Error::new(
                "Each product needs id, W, H, and quantity ≥ 1. Ids must be unique.",
            )));
        }
        if !seen_ids.insert(p.id.as_str()) {
            return Err(JsValue::from(js_sys::Error::new(
                "Each product needs id, W, H, and quantity ≥ 1. Ids must be unique.",
            )));
        }
    }

    let products: Vec<ProductSpec> = req
        .products
        .iter()
        .map(|p| ProductSpec {
            id: p.id.clone(),
            w: p.w,
            h: p.h,
            target: p.target,
            allow_rotation: p.allow_rotation,
        })
        .collect();

    let default_allow = req.allow_rotation.unwrap_or(true);
    let sol = solve_layout(
        req.sheet_w,
        req.sheet_h,
        &products,
        SolveOptions {
            k_max: req.k_max,
            allow_rotation: default_allow,
        },
    )
    .ok_or_else(|| {
        JsValue::from(js_sys::Error::new(
            "No feasible layout for this sheet and k range. Try a larger sheet, fewer items per sheet (lower k via max k), or smaller products.",
        ))
    })?;

    let out = wire_response(
        req.sheet_w,
        req.sheet_h,
        sol,
        &req.products,
        default_allow,
        1.25,
    );
    serde_json::to_string(&out).map_err(|e| {
        JsValue::from(js_sys::Error::new(&format!("Serialize error: {e}")))
    })
}
