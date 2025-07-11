use kurbo::curve_intersections::{curve_curve_intersections, CurveIntersectionFlags};
use kurbo::{Affine, CubicBez, Point, Shape};

use rand::Rng;

fn rand_mono_cubic() -> CubicBez {
    let mut rnd = rand::thread_rng();
    let p0 = Point::new(0., 0.);
    let p1 = Point::new(rnd.gen_range(-1.0..1.), rnd.gen_range(0.0..1.));
    let p2 = Point::new(rnd.gen_range(-1.0..1.), rnd.gen_range(0.0..1.));
    let p3 = Point::new(0., 1.);
    CubicBez::new(p0, p1, p2, p3)
}

fn print_svg(c: CubicBez) {
    let a = Affine::new([500., 0., 0., 500., 550., 10.]);
    let p = (a * c).to_path(1e-9).to_svg();
    println!("  <path d=\"{}\" fill=\"none\" stroke=\"#000\" />", p);
}

fn main() {
    //let c0 = CubicBez::new((0., 0.), (40., 20.), (20., 80.), (100., 100.));
    //let c1 = CubicBez::new((100., 0.), (70., 20.), (80., 70.), (0., 100.));
    for _ in 0..1_000_000 {
        let c0 = rand_mono_cubic();
        let c1 = rand_mono_cubic();
        let start = std::time::Instant::now();
        let intersections = curve_curve_intersections(&c0, &c1, CurveIntersectionFlags::NONE, 1e-6);
        let elapsed = start.elapsed();
        if elapsed > std::time::Duration::from_micros(1_000) {
            println!("{:?} {:?}", c0, c1);
            print_svg(c0);
            print_svg(c1);
            println!("intersections: {:?}", intersections);
            println!("elapsed: {:?}", start.elapsed());
        }
    }
}
