use ctor_lite::{ctor, dtor};
use macro_rules_attribute::apply;
use std::sync::atomic::{AtomicBool, Ordering};

static INITED: AtomicBool = AtomicBool::new(false);
static INITED_2: AtomicBool = AtomicBool::new(false);

ctor! {
    /// Doc comment.
    unsafe fn foo() {
        INITED.store(true, Ordering::SeqCst);
    }
}

/// We need to support more than one of these.
#[apply(ctor!)]
unsafe fn bar() {
    INITED_2.store(true, Ordering::SeqCst);
}

ctor! {
    unsafe static INITED_3: usize = 0xDEAD;
}

dtor! {
    unsafe fn run_at_exit() {
        let stderr = unsafe {
            rustix::stdio::stderr()
        };

        rustix::io::write(stderr, b"Grep for this string at exit! 0123456789").ok();
    }
}

#[test]
fn everything_is_initialized() {
    assert!(INITED.load(Ordering::SeqCst));
    assert!(INITED_2.load(Ordering::SeqCst));
    assert_eq!(*INITED_3, 0xDEAD);
}
