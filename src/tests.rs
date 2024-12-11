use std::num::NonZeroUsize;

use crate::{Arena, Index};

#[derive(Debug, PartialEq, Eq)]
pub struct Item(usize);

#[test]
fn test_arena_get_out_of_bounds() {
    let mut arena = Arena::<Item>::new();
    let index1 = arena.push(Item(7));
    let index2 = arena.push(Item(11));
    let index_bad = Index::from_parts(99, NonZeroUsize::new(1).unwrap());

    assert_eq!(arena.get(index1), Some(&Item(7)));
    assert_eq!(arena.get(index2), Some(&Item(11)));
    assert_eq!(arena.get(index_bad), None);
}
