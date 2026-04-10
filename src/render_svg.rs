//! Port of `Layout/src/renderSvg.ts` — SVG string for one sheet preview.

use crate::guillotine::PlacedRect;

pub struct SvgOptions {
    pub stroke_width: f64,
    pub show_labels: bool,
    pub title: String,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            show_labels: true,
            title: String::new(),
        }
    }
}

const DEFAULT_PALETTE: &[&str] = &[
    "#4C6EF5", "#51CF66", "#FF922B", "#CC5DE8", "#22B8CF", "#F06595",
];

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// SVG for a sheet layout (origin top-left, y down).
#[must_use]
pub fn render_layout_svg(
    sheet_w: i32,
    sheet_h: i32,
    rects: &[PlacedRect],
    options: &SvgOptions,
) -> String {
    let sw = options.stroke_width;
    let show_labels = options.show_labels;
    let pad = 8;
    let title_dy = if options.title.is_empty() { 0 } else { 18 };
    let w = sheet_w + 2 * pad;
    let h = sheet_h + 2 * pad + title_dy;
    let title = &options.title;

    let mut id_to_index: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut next_idx = 0usize;
    for r in rects {
        let pid = r.product_id.as_str();
        id_to_index.entry(pid).or_insert_with(|| {
            let i = next_idx;
            next_idx += 1;
            i
        });
    }

    let mut parts: Vec<String> = Vec::new();
    parts.push("<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string());
    parts.push(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">"));
    parts.push(format!(
        "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"#f8f9fa\"/>"));
    if !title.is_empty() {
        parts.push(format!(
            "<text x=\"{pad}\" y=\"16\" font-size=\"12\" font-family=\"system-ui,sans-serif\">{}</text>",
            escape_xml(title)
        ));
    }
    parts.push(format!(
        "<g transform=\"translate({},{})\">",
        pad,
        pad + title_dy
    ));

    parts.push(format!(
        "<rect x=\"0\" y=\"0\" width=\"{sheet_w}\" height=\"{sheet_h}\" fill=\"white\" stroke=\"#333\" stroke-width=\"{sw}\"/>"));

    for r in rects {
        let idx = *id_to_index.get(r.product_id.as_str()).unwrap_or(&0);
        let fill = DEFAULT_PALETTE[idx % DEFAULT_PALETTE.len()];
        let svg_y = r.y;
        parts.push(format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" fill-opacity=\"0.35\" stroke=\"#222\" stroke-width=\"{}\"/>",
            r.x, svg_y, r.w, r.h, fill, sw
        ));
        if show_labels {
            let label_suffix = if r.rotated { " *" } else { "" };
            let label = escape_xml(&format!("{}{}", r.product_id, label_suffix));
            parts.push(format!(
                "<text x=\"{}\" y=\"{}\" font-size=\"11\" font-family=\"system-ui,sans-serif\" fill=\"#111\">{}</text>",
                r.x + 4,
                svg_y + 14,
                label
            ));
        }
    }

    parts.push("</g>".to_string());
    parts.push("</svg>".to_string());
    parts.join("\n")
}
