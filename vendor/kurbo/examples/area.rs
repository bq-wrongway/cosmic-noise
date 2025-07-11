use kurbo::Shape;

fn main() {
    //let path = kurbo::BezPath::from_svg("M0 0Q1 0 1 1L0 1z").unwrap();
    let path = kurbo::BezPath::from_svg("M0 0C1 0 1 0 1 1L0 1z").unwrap();
    let area = path.area();
    println!("{area}");
}
