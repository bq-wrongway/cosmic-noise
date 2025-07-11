// Copyright 2022 the Kurbo Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Test case for checking whether numeric and analytic computations of
//! moments for cubic Béziers match each other.

use kurbo::{common::GAUSS_LEGENDRE_COEFFS_16, CubicBez, ParamCurve, ParamCurveDeriv};

fn main() {
    let c = CubicBez::new(
        (0.0, 547.3395616558455),
        (266.66666666666663, 741.3396067339968),
        (533.3333333333334, -141.33960673399685),
        (800.0, 52.66043834415456),
    );
    let q = c.deriv();
    let numeric = GAUSS_LEGENDRE_COEFFS_16
        .iter()
        .map(|(wi, xi)| {
            let t = 0.5 + xi * 0.5;
            let p = c.eval(t);
            let d = q.eval(t);
            let a = (0.5 * wi) * d.x * p.y;
            p.y * a
        })
        .sum::<f64>();
    println!("{:?} {}", kurbo::simplify::moment_integrals(c), numeric);
}
