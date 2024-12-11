use arena::{Arena, Index};

fn main() {
    let mut arena = Arena::<()>::new();

    let index0 = arena.insert(());
    let index1 = arena.insert(());
    arena.remove(index0);
    let index2 = arena.insert(());

    println!("{:?} {:?} {:?}", index0, index1, index2);

    println!("Index size: {}B", std::mem::size_of::<Index>());
    println!("Option<Index> size: {}B", std::mem::size_of::<Option<Index>>());
}
