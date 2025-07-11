//
// Copyright (c) 2019 KAMADA Ken'ichi.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.
//

use std::collections::HashMap;
use std::fmt::Write as _;
use mutate_once::MutOnce;

struct Single {
    attr: MutOnce<Vec<u32>>,
}

impl Single {
    fn new() -> Self {
        Self { attr: MutOnce::new(Vec::new()) }
    }

    fn attr(&self) -> &[u32] {
        if !self.attr.is_fixed() {
            let mut v = self.attr.get_mut();
            (0..5).for_each(|x| v.push(x * 2));
        }
        self.attr.get_ref()
    }
}

#[test]
fn single() {
    let x = Single::new();
    assert_eq!(x.attr(), &[0, 2, 4, 6, 8]);
}

struct Collection {
    hash_map: HashMap<u32, MutOnce<String>>,
}

impl Collection {
    fn new() -> Self {
        let mut hash_map = HashMap::new();
        (0..10).for_each(|x| {
            hash_map.insert(x, MutOnce::new(String::new()));
        });
        Self { hash_map }
    }

    fn get(&self, key: u32) -> Option<&str> {
        self.hash_map.get(&key).map(|mo| {
            if !mo.is_fixed() {
                let mut v = mo.get_mut();
                write!(v, "value at {}", key).unwrap();
            }
            mo.get_ref().as_str()
        })
    }
}

#[test]
fn collection() {
    let x = Collection::new();
    assert_eq!(x.get(0), Some("value at 0"));
    assert_eq!(x.get(9), Some("value at 9"));
    assert_eq!(x.get(10), None);
}
