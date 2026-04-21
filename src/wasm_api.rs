//! Minimal WASM surface: JSON in → JSON out (layout + SVG).

use crate::layout_search::{ProductSpec, SolveOptions, solve_layout};
use crate::render_svg::{SheetLocale, SvgOptions, format_sheet_preview_title, render_layout_svg};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolveLayoutRequest {
    pub sheet_w: i32,
    pub sheet_h: i32,
    pub products: Vec<ProductSpecWire>,
    /// Upper bound on the integer scale **k** used to build proportional per-sheet
    /// piece counts (`counts[i] = k * pattern[i]`). Omitted uses 500 in the solver,
    /// combined with a sheet-area cap and an overall maximum of 2000.
    #[serde(default)]
    pub k_max: Option<i32>,
    #[serde(default)]
    pub allow_rotation: Option<bool>,
    /// `"en"` (default) or `"ua"` / `"uk"` — affects the SVG sheet summary line only.
    #[serde(default, alias = "lang", alias = "uiLang", alias = "uiLocale")]
    pub locale: Option<String>,
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
    locale: SheetLocale,
) -> String {
    let per_page: String = products
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{}×{}", p.id, sol.counts_per_page[i]))
        .collect::<Vec<_>>()
        .join(", ");
    format_sheet_preview_title(
        sheet_w,
        sheet_h,
        sol.k,
        &per_page,
        sol.pages,
        sol.utilization,
        sol.pack.mode.as_str(),
        default_allow_rot,
        locale,
    )
}

fn wire_response(
    sheet_w: i32,
    sheet_h: i32,
    sol: crate::layout_search::LayoutSolution,
    products: &[ProductSpecWire],
    default_allow_rot: bool,
    stroke_width: f64,
    locale: SheetLocale,
) -> SolveLayoutResponse {
    let title = build_title(sheet_w, sheet_h, &sol, products, default_allow_rot, locale);
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
    let req: SolveLayoutRequest = serde_json::from_str(input)
        .map_err(|e| JsValue::from(js_sys::Error::new(&format!("Invalid JSON: {e}"))))?;

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
    let locale = req
        .locale
        .as_deref()
        .map(SheetLocale::from_lang_code)
        .unwrap_or_default();
    let sol = solve_layout(
        req.sheet_w,
        req.sheet_h,
        &products,
        &SolveOptions {
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
        locale,
    );
    serde_json::to_string(&out)
        .map_err(|e| JsValue::from(js_sys::Error::new(&format!("Serialize error: {e}"))))
}

#[cfg(test)]
mod request_tests {
    use super::SolveLayoutRequest;

    #[test]
    fn json_locale_camelcase_parses() {
        let j = r#"{"sheetW":1,"sheetH":1,"products":[{"id":"a","w":1,"h":1,"target":1}],"locale":"ua"}"#;
        let r: SolveLayoutRequest = serde_json::from_str(j).expect("deserialize");
        assert_eq!(r.locale.as_deref(), Some("ua"));
    }

    #[test]
    fn json_lang_alias_parses() {
        let j = r#"{"sheetW":1,"sheetH":1,"products":[{"id":"a","w":1,"h":1,"target":1}],"lang":"UA"}"#;
        let r: SolveLayoutRequest = serde_json::from_str(j).expect("deserialize");
        assert_eq!(r.locale.as_deref(), Some("UA"));
    }
}
