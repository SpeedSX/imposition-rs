use layout_rs::{
    compute_proportion_pattern, expand_instances, pack_multi_stage, pages_for_k, solve_layout,
    verify_two_stage, PackOptions, ProductSpec, SolveOptions,
};

#[test]
fn i1_motivating_minimal_p() {
    let products = vec![
        ProductSpec {
            id: "A".into(),
            w: 100,
            h: 300,
            target: 1000,
            allow_rotation: None,
        },
        ProductSpec {
            id: "B".into(),
            w: 70,
            h: 100,
            target: 2000,
            allow_rotation: None,
        },
    ];
    let w = 700;
    let h = 600;
    let k_max = 40;
    let targets: Vec<i32> = products.iter().map(|p| p.target).collect();
    let pat = compute_proportion_pattern(&targets).unwrap();
    let sol = solve_layout(
        w,
        h,
        &products,
        &SolveOptions {
            k_max: Some(k_max),
            allow_rotation: true,
        },
    )
    .expect("solution");
    let implied_p: i32 = targets
        .iter()
        .zip(sol.counts_per_page.iter())
        .map(|(&t, &c)| (t + c - 1) / c)
        .max()
        .unwrap();
    assert_eq!(sol.pages, implied_p);
    if sol.k > 0 {
        assert_eq!(sol.pages, pages_for_k(&targets, &pat, sol.k).unwrap());
    }
    for i in 0..products.len() {
        assert!(sol.counts_per_page[i] * sol.pages >= products[i].target);
    }
    assert!(verify_two_stage(&sol.pack.rects, w, h));
}

#[test]
fn i2_fallback_lower_k() {
    let products = vec![
        ProductSpec {
            id: "A".into(),
            w: 40,
            h: 40,
            target: 2,
            allow_rotation: None,
        },
        ProductSpec {
            id: "B".into(),
            w: 40,
            h: 40,
            target: 4,
            allow_rotation: None,
        },
    ];
    let sol = solve_layout(
        80,
        80,
        &products,
        &SolveOptions {
            k_max: Some(100),
            allow_rotation: true,
        },
    )
    .expect("solution");
    assert_eq!(sol.k, 1);
    assert_eq!(sol.pages, 2);
    assert_eq!(sol.counts_per_page, vec![1, 2]);
}

#[test]
fn g2_01_trivial_pack() {
    let inst = expand_instances(&[layout_rs::SkuRow {
        id: "a".into(),
        w: 50,
        h: 50,
        count: 1,
        allow_rotation: None,
    }]);
    let packs = pack_multi_stage(100, 100, &inst, &PackOptions::default());
    assert!(!packs.is_empty());
    let r = &packs[0].rects;
    assert_eq!(layout_rs::total_placed_area(r), 2500);
    assert!((layout_rs::total_placed_area(r) as f64 / 10_000.0 - 0.25).abs() < 1e-9);
}

#[test]
fn g2_03_rotation_required() {
    let inst = expand_instances(&[layout_rs::SkuRow {
        id: "a".into(),
        w: 40,
        h: 30,
        count: 1,
        allow_rotation: None,
    }]);
    let packs = pack_multi_stage(30, 40, &inst, &PackOptions::default());
    assert!(!packs.is_empty());
    let pr = &packs[0].rects[0];
    assert!(pr.rotated);
    assert_eq!(pr.w, 30);
    assert_eq!(pr.h, 40);
}
