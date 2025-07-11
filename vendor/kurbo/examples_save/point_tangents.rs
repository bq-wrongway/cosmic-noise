use kurbo::{CubicBez, ParamCurve, Point, Shape};

fn main() {
    println!("<svg width='800' height='600' xmlns='http://www.w3.org/2000/svg'>");
    let c = CubicBez::new((200., 200.), (200., 145.), (155., 100.), (100., 100.));
    println!(
        "  <path d='{}' fill='none' stroke='#000'/>",
        c.to_path(1e-9).to_svg()
    );
    let p = Point::new(190., 110.);
    for t in c.tangents_to_point(p) {
        let p1 = c.eval(t);
        println!(
            "  <line x1='{}' y1='{}' x2='{}' y2='{}' stroke='#008'/>",
            p.x, p.y, p1.x, p1.y
        );
    }
    println!("</svg>");
}
