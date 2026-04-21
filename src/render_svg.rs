//! Port of `Layout/src/renderSvg.ts` — SVG string for one sheet preview.

use crate::guillotine::PlacedRect;

/// Language for the one-line summary drawn above the sheet in [`render_layout_svg`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetLocale {
    #[default]
    En,
    Ua,
}

impl SheetLocale {
    #[must_use]
    pub fn from_lang_code(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "ua" | "uk" => Self::Ua,
            _ => Self::En,
        }
    }

    fn mixed_k_label(self) -> &'static str {
        match self {
            Self::En => "— (mixed)",
            Self::Ua => "— (змішано)",
        }
    }

    fn rotation_sheet_default(self, on: bool) -> &'static str {
        match (self, on) {
            (Self::En, true) => "on",
            (Self::En, false) => "off",
            (Self::Ua, true) => "увімк.",
            (Self::Ua, false) => "вимк.",
        }
    }
}

/// Builds the SVG title line (sheet stats) for the given locale.
#[must_use]
pub fn format_sheet_preview_title(
    sheet_w: i32,
    sheet_h: i32,
    k: i32,
    per_page: &str,
    pages: i32,
    utilization: f64,
    pack_mode: &str,
    default_allow_rotation: bool,
    locale: SheetLocale,
) -> String {
    let k_label = if k > 0 {
        k.to_string()
    } else {
        locale.mixed_k_label().to_string()
    };
    let rot = locale.rotation_sheet_default(default_allow_rotation);
    match locale {
        SheetLocale::En => format!(
            "Wp×Hp {sheet_w}×{sheet_h} · k={k_label} · {per_page} · P={pages} sheets · util {:.1}% · mode {pack_mode} · sheet default {rot}",
            utilization * 100.0
        ),
        SheetLocale::Ua => format!(
            "Розмір {sheet_w}×{sheet_h} · k={k_label} · {per_page} · Листів: {pages} · Використання: {:.1}% · Режим: {pack_mode} · Поворот за замовч.: {rot}",
            utilization * 100.0
        ),
    }
}

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
        "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"#f8f9fa\"/>"
    ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_title_en_mixed_k_and_rotation() {
        let t = format_sheet_preview_title(
            700,
            600,
            0,
            "A×2, B×3",
            5,
            0.73,
            "VH",
            true,
            SheetLocale::En,
        );
        assert!(t.contains("700×600"));
        assert!(t.contains("(mixed)"));
        assert!(t.contains("P=5 sheets"));
        assert!(t.contains("util 73.0%"));
        assert!(t.contains("sheet default on"));
    }

    #[test]
    fn sheet_title_ua_positive_k() {
        let t = format_sheet_preview_title(
            100,
            80,
            12,
            "X×1",
            3,
            0.5,
            "HV",
            false,
            SheetLocale::Ua,
        );
        assert!(t.contains("Розмір 100×80"));
        assert!(t.contains("k=12"));
        assert!(t.contains("Листів: 3"));
        assert!(t.contains("Використання: 50.0%"));
        assert!(t.contains("Режим: HV"));
        assert!(t.contains("вимк."));
        assert!(!t.contains("(змішано)"));
    }

    #[test]
    fn sheet_locale_from_code() {
        assert_eq!(SheetLocale::from_lang_code("en"), SheetLocale::En);
        assert_eq!(SheetLocale::from_lang_code("UA"), SheetLocale::Ua);
        assert_eq!(SheetLocale::from_lang_code("uk"), SheetLocale::Ua);
    }

    #[test]
    fn render_svg_escapes_xml_in_title() {
        let rects = vec![PlacedRect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
            product_id: "A".into(),
            rotated: false,
        }];
        let svg = render_layout_svg(
            100,
            100,
            &rects,
            &SvgOptions {
                stroke_width: 1.0,
                show_labels: true,
                title: "R&D \"co\" <tag>".into(),
            },
        );
        assert!(svg.contains("&amp;"));
        assert!(svg.contains("&quot;"));
        assert!(svg.contains("&lt;"));
    }

    #[test]
    fn render_svg_embeds_ukrainian_sheet_title() {
        let title = format_sheet_preview_title(
            50,
            40,
            1,
            "P×2",
            1,
            0.9,
            "VH",
            true,
            SheetLocale::Ua,
        );
        let svg = render_layout_svg(
            50,
            40,
            &[],
            &SvgOptions {
                stroke_width: 1.0,
                show_labels: false,
                title,
            },
        );
        assert!(svg.contains("Листів"));
        assert!(svg.contains("Використання"));
    }
}
