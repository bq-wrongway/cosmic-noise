// Copyright 2023 the Kurbo Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use kurbo::{stroke, BezPath, Cap, Join, Shape, Stroke};

fn main() {
    let d = "M100 100 Q150 150 190 110L 50 50";
    let p = BezPath::from_svg(d).unwrap();
    let style = Stroke::new(5.0)
        .with_caps(Cap::Butt)
        .with_join(Join::Round)
        .with_dashes(15.0, &[10.0, 1.0, 2.0]);
    let s = stroke(p, &style, 1e-1);
    println!("{}", s.to_svg());
    println!("area = {}", s.area())
    /*
    let mut dashed = BezPath::new();
    for el in kurbo::DashIterator::new(p.elements().iter().cloned(), &[10.0]) {
        println!("{el:?}");
        dashed.push(el);
    }
    let s = stroke(dashed, &style, 1e-1);
    println!("{}", s.to_svg());
    */
}
