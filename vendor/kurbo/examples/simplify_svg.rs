// Adapted from code provided by @rachael-wang

use std::{env, fs};

use kurbo::{
    simplify::{SimplifyOptLevel, SimplifyOptions},
    PathSeg,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let svg_contents = fs::read_to_string(&args[1]).unwrap();
    let curve = kurbo::BezPath::from_svg(&svg_contents).expect("SVG parse error");

    println!("<svg width='800' height='640' xmlns='http://www.w3.org/2000/svg'>");
    println!("<g transform='translate(0 100) scale(8)'>");
    let width = 0.05;
    let is_redblue = true;
    let (source, approx, sw) = if is_redblue {
        ("#008", "#800", 0.1)
    } else {
        ("#4a4", "#ff8", (width * 2.0f64).max(0.3))
    };
    println!(
        "  <path d='{}' stroke='{source}' fill='none' stroke-width='{sw}'/>",
        curve.to_svg(),
    );
    if !is_redblue {
        println!(
            "  <path d='{}' stroke='#006' fill='none' stroke-width='0.1'/>",
            curve.to_svg()
        );
    }

    let options = SimplifyOptions::default().opt_level(SimplifyOptLevel::Optimize);
    let simplified_path = kurbo::simplify::simplify_bezpath(curve, width, &options);

    println!(
        "  <path d='{}' stroke='{approx}' fill='none' stroke-width='0.1'/>",
        simplified_path.to_svg()
    );
    for seg in simplified_path.segments() {
        if let PathSeg::Cubic(c) = seg {
            println!(
                "  <circle cx='{}' cy='{}' r='0.25' fill='#444'/>",
                c.p0.x, c.p0.y
            );
            println!(
                "  <line x1='{}' y1='{}' x2='{}' y2='{}' stroke='#888' stroke-width='0.1'/>",
                c.p0.x, c.p0.y, c.p1.x, c.p1.y
            );
            println!(
                "  <line x1='{}' y1='{}' x2='{}' y2='{}' stroke='#888' stroke-width='0.1'/>",
                c.p3.x, c.p3.y, c.p2.x, c.p2.y
            );
        }
    }
    println!("</g>");
    println!("</svg>");
}
