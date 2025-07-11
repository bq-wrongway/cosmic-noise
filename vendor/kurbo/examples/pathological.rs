use kurbo::{offset::CubicOffset, CubicBez, PathEl};

fn pathological_curves() {
    let curve = CubicBez {
        p0: (-1236.3746269978635, 152.17981429574826).into(),
        p1: (-1175.18662093517, 108.04721798590596).into(),
        p2: (-1152.142883879584, 105.76260301083356).into(),
        p3: (-1151.842639804639, 105.73040758939104).into(),
    };
    let offset = 3603.7267536453924;
    let accuracy = 0.1;
    let offset_path = CubicOffset::new(curve, offset);
    let path = kurbo::fit_to_bezpath_opt(&offset_path, accuracy);
    println!("{}", path.to_svg());
    assert!(matches!(path.iter().next(), Some(PathEl::MoveTo(_))));
}

fn main() {
    pathological_curves();
}
