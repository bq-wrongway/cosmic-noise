use kurbo::{fit_to_bezpath, offset::CubicOffset, Affine, CubicBez};

/*
fn cusp_ramp() {
    let dim = 0.003;
    for i in -10..=10 {
        let d = i as f64 / 10.0;
        let c = CubicBez::new((0., 0.), (100. + d, 100. + d), (-d, 100. + d), (100., 0.));
        println!("{:?}", c.detect_cusp(dim));
    }
}

fn random_cusp_stuff() {
    let dim = 0.1;
    let c = CubicBez::new((0., 0.), (100., 100.), (0., 100.), (100., 0.));
    c.detect_cusp(dim);
    let d = 0.5;
    let c = CubicBez::new((0., 0.), (100. + d, 100. + d), (-d, 100. + d), (100., 0.));
    c.detect_cusp(dim);
    c.subsegment(0.4..0.6).detect_cusp(dim);
    c.subsegment(0.45..0.55).detect_cusp(dim);
    (Affine::skew(0.1, 0.0) * c).detect_cusp(dim);
    (Affine::scale_non_uniform(1.0, 2.0) * c).detect_cusp(dim);
}

fn regularize_cascade() {
    let d = 10.0;
    let c = CubicBez::new((0., 0.), (100. + d, 100. + d), (-d, 100. + d), (100., 0.));
    println!("<svg width='600' height='600' xmlns='http://www.w3.org/2000/svg'>");
    for i in 1..= 10 {
        let d = i as f64 / 1.0;
        let cr = c.regularize(d);
        let a = Affine::translate((10.0, 50.0 + 20.0 * i as f64));
        println!("  <path d='{}' stroke='#000' stroke-width='0.2' fill='none' />", (a * cr.to_path(1e-3)).to_svg());
    }
    println!("</svg>")
}
*/

fn near_cusp_offsets() {
    println!("<svg width='600' height='600' xmlns='http://www.w3.org/2000/svg'>");
    for i in -10..=10 {
        let d = i as f64 / 1000.0;
        let c = CubicBez::new((0., 0.), (100. + d, 100. + d), (-d, 100. + d), (100., 0.));
        let co = CubicOffset::new_regularized(c, -1.0, 0.1);
        let path = fit_to_bezpath(&co, 0.1);
        let a = Affine::translate((10.0, 250.0 + 20.0 * i as f64));
        println!(
            "  <path d='{}' stroke='#000' stroke-width='0.2' fill='none' />",
            (a * path).to_svg()
        );
    }
    println!("</svg>")
}

fn main() {
    near_cusp_offsets();
}
