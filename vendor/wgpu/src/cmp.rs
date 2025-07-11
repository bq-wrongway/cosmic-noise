//! We need to impl PartialEq, Eq, PartialOrd, Ord, and Hash for all handle types in wgpu.
//!
//! For types that have some already-unique property, we can use that property to implement these traits.
//!
//! For types (like WebGPU) that don't have such a property, we generate an identifier and use that.

use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    inner: NonZeroU64,
}

impl Identifier {
    pub fn create() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        // Safety: Will take 7000+ years of constant incrementing to overflow. It's fine.
        let inner = unsafe { NonZeroU64::new_unchecked(id) };
        Self { inner }
    }
}

/// Implements PartialEq, Eq, PartialOrd, Ord, and Hash for a type by proxying the operations to a single field.
///
/// ```ignore
/// impl_eq_ord_hash_proxy!(MyType => .field);
/// ```
macro_rules! impl_eq_ord_hash_proxy {
    ($type:ty => $($access:tt)*) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                self $($access)* == other $($access)*
            }
        }

        impl Eq for $type {}

        impl PartialOrd for $type {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $type {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self $($access)*.cmp(&other $($access)*)
            }
        }

        impl std::hash::Hash for $type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self $($access)*.hash(state)
            }
        }
    };
}

/// Implements PartialEq, Eq, PartialOrd, Ord, and Hash for a type by comparing the addresses of the Arcs.
///
/// ```ignore
/// impl_eq_ord_hash_arc_address!(MyType => .field);
/// ```
macro_rules! impl_eq_ord_hash_arc_address {
    ($type:ty => $($access:tt)*) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                let address_left = std::sync::Arc::as_ptr(&self $($access)*);
                let address_right = std::sync::Arc::as_ptr(&other $($access)*);

                address_left == address_right
            }
        }

        impl Eq for $type {}

        impl PartialOrd for $type {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $type {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                let address_left = std::sync::Arc::as_ptr(&self $($access)*);
                let address_right = std::sync::Arc::as_ptr(&other $($access)*);

                address_left.cmp(&address_right)
            }
        }

        impl std::hash::Hash for $type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                let address = std::sync::Arc::as_ptr(&self $($access)*);
                address.hash(state)
            }
        }
    };
}

pub(crate) use {impl_eq_ord_hash_arc_address, impl_eq_ord_hash_proxy};
