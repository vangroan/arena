use crate::{Arena, Index};

#[test]
fn test_push() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");

    assert!(!arena.is_empty());
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(index0), Some(&"Foo"));
}

#[test]
fn test_arena_insert_push() {
    let mut arena = Arena::new();
    let index0 = arena.insert("Foo");
    let index1 = arena.insert("Bar");
    let index2 = arena.insert("Baz");

    assert!(!arena.is_empty());
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.get(index0), Some(&"Foo"));
    assert_eq!(arena.get(index1), Some(&"Bar"));
    assert_eq!(arena.get(index2), Some(&"Baz"));
    assert_ne!(index0, index1);
    assert_ne!(index1, index2);
    assert_ne!(index2, index0);
}

#[test]
fn test_remove() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    arena.remove(index0);

    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
#[should_panic]
fn test_remove_out_of_bounds() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index_bad = Index::from_parts(99, index0.generation.get());

    arena.remove(index_bad);
}

#[test]
fn test_remove_repeat() {
    let mut arena = Arena::new();

    let index0 = arena.push("Foo");
    arena.remove(index0);

    let index1 = arena.push("Bar");
    arena.remove(index1);

    arena.remove(index0);
    arena.remove(index1);

    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), None);
    assert!(arena.is_empty());
}

#[test]
fn test_remove_recycle() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");

    arena.remove(index0);
    let index2 = arena.insert("Baz");

    assert_eq!(index0.slot, index2.slot);
    assert_ne!(index0.generation, index2.generation);
    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), Some(&"Bar"));
    assert_eq!(arena.get(index2), Some(&"Baz"));
}

#[test]
fn test_take() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");
    assert_eq!(arena.len(), 2);

    // take first item
    assert_eq!(arena.take(index0).unwrap(), "Foo");
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), Some(&"Bar"));

    // take first item again
    assert_eq!(arena.take(index0), None);
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), Some(&"Bar"));

    // recycle first item
    let index2 = arena.insert("Baz");
    assert_eq!(index0.slot, index2.slot);
    assert_ne!(index0.generation, index2.generation);
    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), Some(&"Bar"));
    assert_eq!(arena.get(index2), Some(&"Baz"));
}

#[test]
fn test_replace_occupied() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");

    let (index2, foo) = arena.replace(index0, "Baz");
    assert_eq!(arena.len(), 2);
    assert_eq!(index0.slot, index2.slot);
    assert_ne!(index0.generation, index2.generation);

    assert_eq!(foo, Some("Foo"));
    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), Some(&"Bar"));
    assert_eq!(arena.get(index2), Some(&"Baz"));
}

#[test]
fn test_replace_vacant() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");
    arena.remove(index0);

    let (index2, foo) = arena.replace(index0, "Baz");
    assert_eq!(arena.len(), 2);
    assert_eq!(index0.slot, index2.slot);
    assert_ne!(index0.generation, index2.generation);

    assert_eq!(foo, None);
    assert_eq!(arena.get(index0), None);
    assert_eq!(arena.get(index1), Some(&"Bar"));
    assert_eq!(arena.get(index2), Some(&"Baz"));
}

#[test]
fn test_arena_get_out_of_bounds() {
    let mut arena = Arena::new();
    let index0 = arena.push("Foo");
    let index1 = arena.push("Bar");
    let index_bad = Index::from_parts(99, 1);

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
