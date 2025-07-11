use kurbo::{Affine, CubicBez, ParamCurveDeriv, Shape};

fn hodo1(d: f64, x: f64) {
    let c = CubicBez::new((0., 0.), (100. + d, 100. + d), (-d, 100. + d), (100., 0.));
    let a = Affine::translate((x, 10.0));
    println!(
        "  <path d='{}' stroke='#000' fill='none'/>",
        (a * c).to_path(1e-9).to_svg()
    );
    let x0 = x;
    let x1 = x + 100.0;
    let xm = 0.5 * (x0 + x1);
    let y0 = 100.0;
    let y1 = 200.0;
    let ym = 0.5 * (y0 + y1);
    let q = Affine::translate((xm, ym)) * Affine::scale(0.15) * c.deriv();
    println!("  <line x1='{xm}' y1='{y0}' x2='{xm}' y2='{y1}' stroke='#888' fill='none'/>");
    println!("  <line x1='{x0}' y1='{ym}' x2='{x1}' y2='{ym}' stroke='#888' fill='none'/>");
    println!(
        "  <path d='{}' stroke='#44f' fill='none'/>",
        q.to_path(1e-9).to_svg()
    );
}

fn main() {
    println!("<svg width='400' height='220' xmlns='http://www.w3.org/2000/svg'>");
    hodo1(-10.0, 10.0);
    hodo1(0.0, 150.0);
    hodo1(10.0, 290.0);
    println!("</svg>")
}
