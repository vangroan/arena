use std::num::NonZeroUsize;

use crate::{Arena, Index};

#[test]
fn test_arena_get_out_of_bounds() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");
    let index_bad = Index::from_parts(99, NonZeroUsize::new(1).unwrap());

    assert_eq!(arena.get(index0), Some(&"Foo"));
    assert_eq!(arena.get(index1), Some(&"Bar"));
    assert_eq!(arena.get(index_bad), None);
}

#[test]
fn test_arena_get2_mut() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");

    let (foo, bar) = arena.get2_mut(index0, index1);
    assert_eq!(foo, Some(&mut "Foo"));
    assert_eq!(bar, Some(&mut "Bar"));
}
