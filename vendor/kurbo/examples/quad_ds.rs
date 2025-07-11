use kurbo::{ParamCurve, Point, QuadBez};
use rand::{thread_rng, Rng};

fn random_pt() -> Point {
    let mut rng = thread_rng();
    Point::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0))
}

fn random_quad() -> QuadBez {
    QuadBez::new(random_pt(), random_pt(), random_pt())
}

fn pt_ds(p: Point) -> f64 {
    let h = p.to_vec2().hypot2();
    h * h.sqrt()
}

fn eval_ds(q: &QuadBez, t: f64) -> f64 {
    pt_ds(q.eval(t))
}

fn one_random() -> (f64, f64) {
    let q = random_quad();
    let y0 = pt_ds(q.p0);
    let y1 = pt_ds(q.p2);
    let mut ymin = 0.0;
    let mut ymax = 0.0;
    for (i, (_, xi)) in kurbo::common::GAUSS_LEGENDRE_COEFFS_16.iter().enumerate() {
        let t = 0.5 + 0.5 * xi;
        let y = eval_ds(&q, t);
        let dy = y - y0 - t * (y1 - y0);
        let sdy = dy / (t - t * t);
        if i == 0 || sdy < ymin {
            ymin = sdy;
        }
        if i == 0 || sdy > ymax {
            ymax = sdy;
        }
    }
    const N: usize = 100;
    let mut min = 0.0;
    let mut max = 0.0;
    for i in 1..N {
        let t = i as f64 / N as f64;
        let t = (3.0 - 2.0 * t) * t * t;
        let y = eval_ds(&q, t);
        let lin = y0 + t * (y1 - y0);
        let gmin = lin + ymin * (t - t * t);
        let gmax = lin + ymax * (t - t * t);
        let ratio = (y - gmin) / (gmax - gmin);
        if i == 1 || ratio < min {
            min = ratio;
        }
        if i == 1 || ratio > max {
            max = ratio;
        }
    }
    (min, max)
}

fn main() {
    const N: usize = 100_000_000;
    let mut min = 0.0;
    let mut max = 0.0;
    for i in 0..N {
        let (a, b) = one_random();
        if i == 0 || a < min {
            min = a;
        }
        if i == 0 || b > max {
            max = b;
        }
    }
    println!("{} {}", min, max);
}
