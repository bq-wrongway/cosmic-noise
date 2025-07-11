use kurbo::{BezPath, CubicBez, Shape};

fn main() {
    let c = CubicBez::new((100., 200.), (105., 205.), (400., 205.), (405., 200.));
    let mut bp = BezPath::new();
    c.to_path(1e-9).flatten(1e-2, |el| bp.push(el));
    println!("{}", bp.to_svg())
}
