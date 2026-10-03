use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashSet};
use std::hash::{Hash, Hasher};

use genco::prelude::*;

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

#[test]
fn test_differing_imports() {
    let hash_map = rust::import("std::collections", "HashMap");
    let btree_map = rust::import("std::collections", "BTreeMap");

    let a: rust::Tokens = quote!(let m = $(&hash_map)::new(););
    let b: rust::Tokens = quote!(let m = $(&btree_map)::new(););

    assert_ne!(a, b);
    assert_ne!(a.cmp(&b), Ordering::Equal);
    assert_eq!(a.cmp(&b), a.partial_cmp(&b).unwrap());
    assert_eq!(a.cmp(&b), b.cmp(&a).reverse());

    let set: HashSet<_> = vec![a.clone(), b.clone()].into_iter().collect();
    assert_eq!(set.len(), 2);
    let set: BTreeSet<_> = vec![a, b].into_iter().collect();
    assert_eq!(set.len(), 2);
}

#[test]
fn test_same_imports() {
    let hash_map = rust::import("std::collections", "HashMap");

    let a: rust::Tokens = quote!(let m = $(&hash_map)::new(););
    let b: rust::Tokens = quote!(let m = $(&hash_map)::new(););

    assert_eq!(a, b);
    assert_eq!(a.cmp(&b), Ordering::Equal);
    assert_eq!(hash(&a), hash(&b));
}

#[test]
fn test_register_only() {
    let write = rust::import("std::fmt", "Write");

    let registered: rust::Tokens = quote!($(register(&write)));
    let empty: rust::Tokens = quote!();

    assert!(registered.is_empty());
    assert_ne!(registered, empty);
    assert_ne!(registered.cmp(&empty), Ordering::Equal);
    assert_eq!(registered, quote!($(register(&write))));
    assert_eq!(hash(&registered), hash(&quote!($(register(&write)))));

    let other: rust::Tokens = quote!($(register(rust::import("std::io", "Write"))));
    assert_ne!(registered, other);

    let set: HashSet<_> = vec![registered, empty, other].into_iter().collect();
    assert_eq!(set.len(), 3);
}
