//! Port of `Layout/src/guillotine.ts` — two-stage guillotine packing (V–H / H–V).

#[derive(Debug, Clone)]
pub struct PlacedRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub product_id: String,
    pub rotated: bool,
}

#[derive(Debug, Clone)]
pub struct PackInstance {
    pub product_id: String,
    pub w: i32,
    pub h: i32,
    pub allow_rotation: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackMode {
    Vh,
    Hv,
}

impl PackMode {
    pub fn as_str(self) -> &'static str {
        match self {
            PackMode::Vh => "VH",
            PackMode::Hv => "HV",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackResult {
    pub rects: Vec<PlacedRect>,
    pub mode: PackMode,
    pub utilization: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderSeed {
    Default,
    AreaDesc,
    AreaAsc,
    WidthDesc,
    HeightDesc,
    ProductId,
}

struct ColumnState {
    x: i32,
    w: i32,
    placed: Vec<PlacedRect>,
}

struct RowState {
    y: i32,
    h: i32,
    placed: Vec<PlacedRect>,
}

fn rects_overlap(a: &PlacedRect, b: &PlacedRect) -> bool {
    if a.x + a.w <= b.x || b.x + b.w <= a.x {
        return false;
    }
    if a.y + a.h <= b.y || b.y + b.h <= a.y {
        return false;
    }
    true
}

fn all_inside_sheet(rects: &[PlacedRect], wp: i32, hp: i32) -> bool {
    rects
        .iter()
        .all(|r| r.x >= 0 && r.y >= 0 && r.x + r.w <= wp && r.y + r.h <= hp)
}

fn no_overlaps(rects: &[PlacedRect]) -> bool {
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            if rects_overlap(&rects[i], &rects[j]) {
                return false;
            }
        }
    }
    true
}

/// V–H: columns tile [0,Wp]; within each column, rects share (x,w) and stack from y=0.
#[must_use]
pub fn verify_two_stage_vh(rects: &[PlacedRect], wp: i32, hp: i32) -> bool {
    if rects.is_empty() {
        return true;
    }
    if !all_inside_sheet(rects, wp, hp) || !no_overlaps(rects) {
        return false;
    }

    let key = |r: &PlacedRect| format!("{},{}", r.x, r.w);
    let mut groups: std::collections::HashMap<String, Vec<PlacedRect>> =
        std::collections::HashMap::new();
    for r in rects {
        groups.entry(key(r)).or_default().push(r.clone());
    }

    let mut columns: Vec<(i32, i32, Vec<PlacedRect>)> = Vec::new();
    for mut rs in groups.into_values() {
        let x = rs[0].x;
        let w = rs[0].w;
        for r in &rs {
            if r.x != x || r.w != w {
                return false;
            }
        }
        rs.sort_by_key(|r| r.y);
        let mut y = 0;
        for r in &rs {
            if r.y != y {
                return false;
            }
            y += r.h;
        }
        if y > hp {
            return false;
        }
        columns.push((x, w, rs));
    }

    columns.sort_by_key(|c| c.0);
    if columns[0].0 != 0 {
        return false;
    }
    for i in 0..columns.len() {
        let c = &columns[i];
        if i + 1 < columns.len() {
            let next = &columns[i + 1];
            if c.0 + c.1 != next.0 {
                return false;
            }
        } else if c.0 + c.1 > wp {
            return false;
        }
    }
    true
}

/// H–V: rows tile [0,Hp]; within each row, rects share (y,h) and pack from x=0.
#[must_use]
pub fn verify_two_stage_hv(rects: &[PlacedRect], wp: i32, hp: i32) -> bool {
    if rects.is_empty() {
        return true;
    }
    if !all_inside_sheet(rects, wp, hp) || !no_overlaps(rects) {
        return false;
    }

    let key = |r: &PlacedRect| format!("{},{}", r.y, r.h);
    let mut groups: std::collections::HashMap<String, Vec<PlacedRect>> =
        std::collections::HashMap::new();
    for r in rects {
        groups.entry(key(r)).or_default().push(r.clone());
    }

    let mut rows: Vec<(i32, i32, Vec<PlacedRect>)> = Vec::new();
    for mut rs in groups.into_values() {
        let y = rs[0].y;
        let h = rs[0].h;
        for r in &rs {
            if r.y != y || r.h != h {
                return false;
            }
        }
        rs.sort_by_key(|r| r.x);
        let mut x = 0;
        for r in &rs {
            if r.x != x {
                return false;
            }
            x += r.w;
        }
        if x > wp {
            return false;
        }
        rows.push((y, h, rs));
    }

    rows.sort_by_key(|r| r.0);
    if rows[0].0 != 0 {
        return false;
    }
    for i in 0..rows.len() {
        let row = &rows[i];
        if i + 1 < rows.len() {
            let next = &rows[i + 1];
            if row.0 + row.1 != next.0 {
                return false;
            }
        } else if row.0 + row.1 > hp {
            return false;
        }
    }
    true
}

#[must_use]
pub fn verify_two_stage(rects: &[PlacedRect], wp: i32, hp: i32) -> bool {
    verify_two_stage_vh(rects, wp, hp) || verify_two_stage_hv(rects, wp, hp)
}

#[must_use]
pub fn total_placed_area(rects: &[PlacedRect]) -> i64 {
    rects.iter().map(|r| i64::from(r.w) * i64::from(r.h)).sum()
}

fn clone_placed(r: &PlacedRect) -> PlacedRect {
    r.clone()
}

/// Orientations: native only, or native + 90° swap.
fn rotation_flags(allow_rotation: bool) -> &'static [bool] {
    if allow_rotation {
        &[false, true]
    } else {
        &[false]
    }
}

fn instance_allows_rotation(inst: &PackInstance, default_allow_rotation: bool) -> bool {
    inst.allow_rotation.unwrap_or(default_allow_rotation)
}

fn pack_vh_dfs(
    cols: &mut Vec<ColumnState>,
    rem: &[PackInstance],
    wp: i32,
    hp: i32,
    default_allow_rotation: bool,
) -> Option<Vec<PlacedRect>> {
    if rem.is_empty() {
        let mut out = Vec::new();
        for c in cols.iter() {
            for r in &c.placed {
                out.push(clone_placed(r));
            }
        }
        return Some(out);
    }

    let used_w: i32 = cols.iter().map(|c| c.w).sum();
    let item = &rem[0];
    let rest = &rem[1..];
    let item_rot = instance_allows_rotation(item, default_allow_rotation);

    for &rot in rotation_flags(item_rot) {
        let (ow, oh) = if rot {
            (item.h, item.w)
        } else {
            (item.w, item.h)
        };

        for ci in 0..cols.len() {
            if cols[ci].w != ow {
                continue;
            }
            let y: i32 = cols[ci].placed.iter().map(|r| r.h).sum();
            if y + oh > hp {
                continue;
            }
            let col_x = cols[ci].x;
            let pr = PlacedRect {
                x: col_x,
                y,
                w: ow,
                h: oh,
                product_id: item.product_id.clone(),
                rotated: rot,
            };
            cols[ci].placed.push(pr);
            let out = pack_vh_dfs(cols, rest, wp, hp, default_allow_rotation);
            if out.is_some() {
                return out;
            }
            cols[ci].placed.pop();
        }

        if used_w + ow <= wp {
            let x = used_w;
            let pr = PlacedRect {
                x,
                y: 0,
                w: ow,
                h: oh,
                product_id: item.product_id.clone(),
                rotated: rot,
            };
            cols.push(ColumnState {
                x,
                w: ow,
                placed: vec![pr],
            });
            let out = pack_vh_dfs(cols, rest, wp, hp, default_allow_rotation);
            cols.pop();
            if out.is_some() {
                return out;
            }
        }
    }
    None
}

fn pack_hv_dfs(
    rows: &mut Vec<RowState>,
    rem: &[PackInstance],
    wp: i32,
    hp: i32,
    default_allow_rotation: bool,
) -> Option<Vec<PlacedRect>> {
    if rem.is_empty() {
        let mut out = Vec::new();
        for row in rows.iter() {
            for r in &row.placed {
                out.push(clone_placed(r));
            }
        }
        return Some(out);
    }

    let used_h: i32 = rows.iter().map(|r| r.h).sum();
    let item = &rem[0];
    let rest = &rem[1..];
    let item_rot = instance_allows_rotation(item, default_allow_rotation);

    for &rot in rotation_flags(item_rot) {
        let (ow, oh) = if rot {
            (item.h, item.w)
        } else {
            (item.w, item.h)
        };

        for ri in 0..rows.len() {
            if rows[ri].h != oh {
                continue;
            }
            let x: i32 = rows[ri].placed.iter().map(|r| r.w).sum();
            if x + ow > wp {
                continue;
            }
            let row_y = rows[ri].y;
            let pr = PlacedRect {
                x,
                y: row_y,
                w: ow,
                h: oh,
                product_id: item.product_id.clone(),
                rotated: rot,
            };
            rows[ri].placed.push(pr);
            let out = pack_hv_dfs(rows, rest, wp, hp, default_allow_rotation);
            if out.is_some() {
                return out;
            }
            rows[ri].placed.pop();
        }

        if used_h + oh <= hp {
            let y = used_h;
            let pr = PlacedRect {
                x: 0,
                y,
                w: ow,
                h: oh,
                product_id: item.product_id.clone(),
                rotated: rot,
            };
            rows.push(RowState {
                y,
                h: oh,
                placed: vec![pr],
            });
            let out = pack_hv_dfs(rows, rest, wp, hp, default_allow_rotation);
            rows.pop();
            if out.is_some() {
                return out;
            }
        }
    }
    None
}

const DFS_INSTANCE_LIMIT: usize = 12;

fn pack_vh_greedy(
    ordered: &[PackInstance],
    wp: i32,
    hp: i32,
    default_allow_rotation: bool,
) -> Option<Vec<PlacedRect>> {
    let mut cols: Vec<ColumnState> = Vec::new();
    for item in ordered {
        let mut placed = false;
        let item_rot = instance_allows_rotation(item, default_allow_rotation);
        for &rot in rotation_flags(item_rot) {
            let (ow, oh) = if rot {
                (item.h, item.w)
            } else {
                (item.w, item.h)
            };
            for col in &mut cols {
                if col.w != ow {
                    continue;
                }
                let y: i32 = col.placed.iter().map(|r| r.h).sum();
                if y + oh > hp {
                    continue;
                }
                col.placed.push(PlacedRect {
                    x: col.x,
                    y,
                    w: ow,
                    h: oh,
                    product_id: item.product_id.clone(),
                    rotated: rot,
                });
                placed = true;
                break;
            }
            if placed {
                break;
            }
            let used_w: i32 = cols.iter().map(|c| c.w).sum();
            if used_w + ow <= wp {
                let x = used_w;
                cols.push(ColumnState {
                    x,
                    w: ow,
                    placed: vec![PlacedRect {
                        x,
                        y: 0,
                        w: ow,
                        h: oh,
                        product_id: item.product_id.clone(),
                        rotated: rot,
                    }],
                });
                placed = true;
                break;
            }
        }
        if !placed {
            return None;
        }
    }
    Some(
        cols.iter()
            .flat_map(|c| c.placed.iter().map(clone_placed))
            .collect(),
    )
}

fn pack_hv_greedy(
    ordered: &[PackInstance],
    wp: i32,
    hp: i32,
    default_allow_rotation: bool,
) -> Option<Vec<PlacedRect>> {
    let mut rows: Vec<RowState> = Vec::new();
    for item in ordered {
        let mut placed = false;
        let item_rot = instance_allows_rotation(item, default_allow_rotation);
        for &rot in rotation_flags(item_rot) {
            let (ow, oh) = if rot {
                (item.h, item.w)
            } else {
                (item.w, item.h)
            };
            for row in &mut rows {
                if row.h != oh {
                    continue;
                }
                let x: i32 = row.placed.iter().map(|r| r.w).sum();
                if x + ow > wp {
                    continue;
                }
                row.placed.push(PlacedRect {
                    x,
                    y: row.y,
                    w: ow,
                    h: oh,
                    product_id: item.product_id.clone(),
                    rotated: rot,
                });
                placed = true;
                break;
            }
            if placed {
                break;
            }
            let used_h: i32 = rows.iter().map(|r| r.h).sum();
            if used_h + oh <= hp {
                let y = used_h;
                rows.push(RowState {
                    y,
                    h: oh,
                    placed: vec![PlacedRect {
                        x: 0,
                        y,
                        w: ow,
                        h: oh,
                        product_id: item.product_id.clone(),
                        rotated: rot,
                    }],
                });
                placed = true;
                break;
            }
        }
        if !placed {
            return None;
        }
    }
    Some(
        rows.iter()
            .flat_map(|r| r.placed.iter().map(clone_placed))
            .collect(),
    )
}

pub fn sort_instances(instances: &[PackInstance], seed: OrderSeed) -> Vec<PackInstance> {
    let mut copy: Vec<PackInstance> = instances.to_vec();
    match seed {
        OrderSeed::AreaDesc => {
            copy.sort_by(|a, b| {
                let aa = a.w * a.h;
                let bb = b.w * b.h;
                bb.cmp(&aa).then_with(|| a.product_id.cmp(&b.product_id))
            });
        }
        OrderSeed::AreaAsc => {
            copy.sort_by(|a, b| {
                let aa = a.w * a.h;
                let bb = b.w * b.h;
                aa.cmp(&bb).then_with(|| a.product_id.cmp(&b.product_id))
            });
        }
        OrderSeed::WidthDesc => {
            copy.sort_by(|a, b| b.w.cmp(&a.w).then_with(|| a.product_id.cmp(&b.product_id)));
        }
        OrderSeed::HeightDesc => {
            copy.sort_by(|a, b| b.h.cmp(&a.h).then_with(|| a.product_id.cmp(&b.product_id)));
        }
        OrderSeed::ProductId => {
            copy.sort_by(|a, b| a.product_id.cmp(&b.product_id));
        }
        OrderSeed::Default => {}
    }
    copy
}

const DEFAULT_SEEDS: &[OrderSeed] = &[
    OrderSeed::Default,
    OrderSeed::AreaDesc,
    OrderSeed::AreaAsc,
    OrderSeed::WidthDesc,
    OrderSeed::HeightDesc,
    OrderSeed::ProductId,
];

/// One SKU line (id, dimensions, how many copies on the sheet).
#[derive(Debug, Clone)]
pub struct SkuRow {
    pub id: String,
    pub w: i32,
    pub h: i32,
    pub count: i32,
    pub allow_rotation: Option<bool>,
}

/// Expand SKU rows into individual instances for packing.
#[must_use]
pub fn expand_instances(products: &[SkuRow]) -> Vec<PackInstance> {
    let mut out = Vec::new();
    for p in products {
        for _ in 0..p.count {
            out.push(PackInstance {
                product_id: p.id.clone(),
                w: p.w,
                h: p.h,
                allow_rotation: p.allow_rotation,
            });
        }
    }
    out
}

fn signature(rects: &[PlacedRect]) -> String {
    let mut parts: Vec<String> = rects
        .iter()
        .map(|r| {
            format!(
                "{},{},{},{},{},{}",
                r.x, r.y, r.w, r.h, r.product_id, r.rotated
            )
        })
        .collect();
    parts.sort();
    parts.join("|")
}

pub struct PackOptions<'a> {
    pub max_solutions: usize,
    pub seeds: &'a [OrderSeed],
    pub allow_rotation: bool,
}

impl Default for PackOptions<'_> {
    fn default() -> Self {
        Self {
            max_solutions: 8,
            seeds: DEFAULT_SEEDS,
            allow_rotation: true,
        }
    }
}

/// Try 2-stage V–H / H–V with optional 90° rotation, multiple item orderings.
#[must_use]
pub fn pack_multi_stage(
    wp: i32,
    hp: i32,
    instances: &[PackInstance],
    options: &PackOptions<'_>,
) -> Vec<PackResult> {
    let max_sol = options.max_solutions;
    let seeds = if options.seeds.is_empty() {
        DEFAULT_SEEDS
    } else {
        options.seeds
    };
    let default_allow_rotation = options.allow_rotation;
    let sheet_area = f64::from(wp) * f64::from(hp);
    let mut seen = std::collections::HashSet::<String>::new();
    let mut results: Vec<PackResult> = Vec::new();

    let try_push = |rects: Vec<PlacedRect>,
                    mode: PackMode,
                    results: &mut Vec<PackResult>,
                    seen: &mut std::collections::HashSet<String>| {
        let sig = signature(&rects);
        if !seen.insert(sig) {
            return;
        }
        let util = total_placed_area(&rects) as f64 / sheet_area;
        results.push(PackResult {
            rects,
            mode,
            utilization: util,
        });
    };

    let use_dfs = instances.len() <= DFS_INSTANCE_LIMIT;

    for seed in seeds.iter().copied() {
        let ordered = sort_instances(instances, seed);
        let vh = if use_dfs {
            pack_vh_dfs(&mut vec![], &ordered, wp, hp, default_allow_rotation)
        } else {
            pack_vh_greedy(&ordered, wp, hp, default_allow_rotation)
        };

        if let Some(ref rects) = vh
            && all_inside_sheet(rects, wp, hp)
            && no_overlaps(rects)
        {
            try_push(rects.clone(), PackMode::Vh, &mut results, &mut seen);
        }

        let ordered2 = sort_instances(instances, seed);
        let hv = if use_dfs {
            pack_hv_dfs(&mut vec![], &ordered2, wp, hp, default_allow_rotation)
        } else {
            pack_hv_greedy(&ordered2, wp, hp, default_allow_rotation)
        };

        if let Some(ref rects) = hv
            && all_inside_sheet(rects, wp, hp)
            && no_overlaps(rects)
        {
            try_push(rects.clone(), PackMode::Hv, &mut results, &mut seen);
        }

        if results.len() >= max_sol {
            return rank_pack_results(results);
        }
    }

    rank_pack_results(results)
}

fn rank_pack_results(mut results: Vec<PackResult>) -> Vec<PackResult> {
    results.sort_by(|a, b| {
        b.utilization
            .partial_cmp(&a.utilization)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

/// Necessary condition: sum of areas fits.
#[must_use]
pub fn trivial_area_feasible(instances: &[PackInstance], wp: i32, hp: i32) -> bool {
    let sum: i64 = instances
        .iter()
        .map(|i| i64::from(i.w) * i64::from(i.h))
        .sum();
    sum <= i64::from(wp) * i64::from(hp)
}
