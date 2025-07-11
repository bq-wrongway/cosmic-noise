use kurbo::{CubicBez, EulerSeg, ParamCurve, ParamCurveDeriv, Point};

// Experiments on the

struct Quintic([Point; 6]);

impl Quintic {
    fn eval(&self, t: f64) -> Point {
        let mt = 1. - t;
        (self.0[0].to_vec2() * mt.powi(5)
            + 5.0 * self.0[1].to_vec2() * mt.powi(4) * t
            + 10.0 * self.0[2].to_vec2() * mt.powi(3) * t * t
            + 10.0 * self.0[3].to_vec2() * mt * mt * t.powi(3)
            + 5.0 * self.0[4].to_vec2() * mt * t.powi(4)
            + self.0[5].to_vec2() * t.powi(5))
        .to_point()
    }

    fn from_euler_seg(es: &EulerSeg) -> Quintic {
        let esd = es.deriv();
        let d0 = esd.eval(0.0).to_vec2();
        let d1 = esd.eval(1.0).to_vec2();
        // Will replace these with symbolic derivs
        let dd0 = 1000. * (esd.eval(0.001).to_vec2() - d0);
        let dd1 = 1000. * (d1 - esd.eval(0.999).to_vec2());
        println!("dd0 = {:?}, dd1 = {:?}", dd0, dd1);
        let p0 = es.start();
        let p5 = es.end();
        let c = [
            p0,
            p0 + 0.2 * d0,
            p0 + 0.4 * d0 + 0.05 * dd0,
            p5 - 0.4 * d1 + 0.05 * dd1,
            p5 - 0.2 * d1,
            p5,
        ];
        Quintic(c)
    }
}

/// A cubic approximation that tries to match the parametrization,
/// using Hermite interpolation
fn es_cubic_approx(es: &EulerSeg) -> CubicBez {
    let esd = es.deriv();
    let d0 = esd.eval(0.0).to_vec2();
    let d1 = esd.eval(1.0).to_vec2();
    let p0 = es.start();
    let p3 = es.end();
    CubicBez::new(p0, p0 + (1. / 3.) * d0, p3 - (1. / 3.) * d1, p3)
}

fn main() {
    let es = EulerSeg::new(Point::ORIGIN, Point::new(1., 0.), -0.2, 0.2);
    let q = Quintic::from_euler_seg(&es);
    //let q = es_cubic_approx(&es);
    for i in 0..=10 {
        let t = i as f64 / 10.;
        println!("{}: {:?} {:?}", i, es.eval(t), q.eval(t));
    }
}
