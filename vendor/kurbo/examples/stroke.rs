use kurbo::{stroke, CubicBez, Shape, Stroke};

fn main() {
    let curve = CubicBez::new((0., -0.01), (128., 128.001), (128., -0.01), (0., 128.001));
    let path = curve.into_path(0.1);
    let stroke_style = Stroke::new(30.);
    let stroked = stroke(path, &stroke_style, &Default::default(), 0.1);
    println!("<svg width='800' height='600' xmlns='http://www.w3.org/2000/svg'>");
    println!(
        "  <path d='{}' stroke='#000' fill='none' stroke-width='1'/>",
        stroked.to_svg()
    );
    println!("</svg>");
}
