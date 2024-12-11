//! Generations use [`NonZeroUsize`] to reduce the size of `Option<Index>`.
use std::iter::Iterator;
use std::num::NonZeroUsize;
use std::slice::Iter as SliceIter;

#[cfg(test)]
mod tests;

/// Generation Arena.
#[derive(Debug, Clone)]
pub struct Arena<T> {
    data: Vec<Entry<T>>,
    generation: NonZeroUsize,
    free_head: Option<usize>,
    count: usize,
}

#[derive(Debug, Clone)]
pub enum Entry<T> {
    Vacant { next: Option<usize> },
    Occupied { generation: NonZeroUsize, item: T },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    generation: NonZeroUsize,
    slot: usize,
}

impl<T> Arena<T> {
    /// Create a new [`Arena`] instance.
    ///
    /// ```
    /// # use arena::Arena;
    /// # struct GameObject;
    /// let mut arena = Arena::<GameObject>::new();
    ///
    /// // or...
    ///
    /// let mut arena: Arena<GameObject> = Arena::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            generation: NonZeroUsize::new(1).unwrap(),
            free_head: None,
            count: 0,
        }
    }

    /// Removes an item from the arena.
    ///
    /// ```
    /// # use arena::Arena;
    /// # let mut arena = Arena::new();
    /// let index = arena.push("Foo");
    /// # assert!(!arena.is_empty());
    /// # assert_eq!(arena.len(), 1);
    ///
    /// arena.remove(index);
    /// # assert!(arena.is_empty());
    /// # assert_eq!(arena.len(), 0);
    /// ```
    ///
    /// # Panic
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: Index) {
        if let Entry::Occupied { generation, .. } = &self.data[index.slot] {
            if index.generation == *generation {
                self.data[index.slot] = Entry::Vacant { next: self.free_head };
                self.free_head = Some(index.slot);
                self.generation = self.generation.saturating_add(1);
                self.count -= 1;
            }
        }
    }

    /// Remove and return the item at the given `index`.
    ///
    /// ```
    /// # use arena::Arena;
    /// # let mut arena = Arena::new();
    /// let index = arena.insert("Foo");
    /// let object = arena.take(index).unwrap();
    /// # assert_eq!(object, "Foo");
    /// # assert_eq!(arena.len(), 0);
    /// # assert!(arena.get(index).is_none());
    /// ```
    ///
    /// # Panic
    ///
    /// Panics if `index` is out of bounds.
    pub fn take(&mut self, index: Index) -> Option<T> {
        let entry = &mut self.data[index.slot];

        if entry.is_occupied() {
            let original = std::mem::replace(entry, Entry::Vacant { next: self.free_head });
            self.generation = self.generation.saturating_add(1);
            self.count -= 1;
            Some(original.unwrap_occupied().1)
        } else {
            None
        }
    }

    /// Appends the item to the end of the arena.
    pub fn push(&mut self, item: T) -> Index {
        let generation = self.generation;
        let pos = self.data.len();
        self.data.push(Entry::Occupied { generation, item });
        self.count += 1;
        Index { generation, slot: pos }
    }

    /// Insert the item into the first free slot.
    ///
    /// ```
    /// # use arena::Arena;
    /// # let mut arena = Arena::new();
    /// # let index0 = arena.push("Foo");
    /// # let index1 = arena.push("Bar");
    /// # arena.remove(index0);
    /// let index = arena.insert("Foo");
    /// # assert_ne!(index, index0);
    /// # assert_eq!(arena.len(), 2);
    /// ```
    pub fn insert(&mut self, item: T) -> Index {
        match self.free_head.take() {
            Some(pos) => {
                let generation = self.generation;
                self.data[pos] = Entry::Occupied { generation, item };
                self.count += 1;
                Index { generation, slot: pos }
            }
            None => self.push(item),
        }
    }

    /// Set the item at the given `index`.
    ///
    /// Returns an index for the new generation, and
    /// optionally returns the existing item if the
    /// slot is occupied.
    ///
    /// ```
    /// # use arena::Arena;
    /// # let mut arena = Arena::new();
    /// let index0 = arena.insert("Foo");
    /// let (index1, original) = arena.replace(index0, "Bar");
    ///
    /// assert_eq!(original, Some("Foo"));
    /// assert_eq!(arena.get(index0), None);
    /// assert_eq!(arena.get(index1), Some(&"Bar"));
    /// # assert_eq!(arena.len(), 1);
    /// ```
    ///
    /// # Panic
    ///
    /// Panics if `index` is out of bounds.
    pub fn replace(&mut self, index: Index, item: T) -> (Index, Option<T>) {
        let entry = &mut self.data[index.slot];

        if entry.is_occupied() {
            let generation = self.generation.saturating_add(1);
            let original = std::mem::replace(entry, Entry::Occupied { generation, item });
            self.generation = generation;
            (
                Index {
                    generation,
                    slot: index.slot,
                },
                Some(original.unwrap_occupied().1),
            )
        } else {
            let generation = self.generation;
            *entry = Entry::Occupied { generation, item };
            self.count += 1;
            (
                Index {
                    generation,
                    slot: index.slot,
                },
                None,
            )
        }
    }

    pub fn set(&mut self, index: Index, item: T) {
        let entry = &mut self.data[index.slot];

        if entry.is_occupied() {
            let generation = self.generation.saturating_add(1);
            let _ = std::mem::replace(entry, Entry::Occupied { generation, item });
            self.generation = generation;
        } else {
            *entry = Entry::Occupied {
                generation: self.generation,
                item,
            };
            self.count += 1;
        }
    }

    /// Return a reference to the item at the given `index`.
    ///
    /// ```
    /// # use arena::Arena;
    /// # struct GameObject { position: [f32; 2] };
    /// # let mut arena = Arena::<GameObject>::new();
    /// let index = arena.push(GameObject { position: [2.0, 3.0] });
    ///
    /// let object = arena.get(index).unwrap();
    /// # assert_eq!(object.position, [2.0, 3.0]);
    /// ```
    pub fn get(&self, index: Index) -> Option<&T> {
        if let Some(Entry::Occupied { generation, item }) = self.data.get(index.slot) {
            if index.generation == *generation {
                return Some(item);
            }
        }

        None
    }

    /// Return a mutable reference to the item at the given `index`.
    ///
    /// ```
    /// # use arena::Arena;
    /// # struct GameObject { position: [f32; 2] };
    /// # let mut arena = Arena::<GameObject>::new();
    /// let index = arena.push(GameObject { position: [2.0, 3.0] });
    ///
    /// let object = arena.get_mut(index).unwrap();
    /// object.position[0] = 7.0;
    /// object.position[1] = 11.0;
    /// # assert_eq!(object.position, [7.0, 11.0]);
    /// # assert_eq!(arena.get_mut(index).unwrap().position, [7.0, 11.0])
    /// ```
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        if let Some(Entry::Occupied { generation, item }) = self.data.get_mut(index.slot) {
            if index.generation == *generation {
                return Some(item);
            }
        }

        None
    }

    /// # Panic
    ///
    /// Panics if the two indices point to the same slot.
    pub fn get2_mut(&mut self, a: Index, b: Index) -> (Option<&mut T>, Option<&mut T>) {
        assert_ne!(a.slot, b.slot);

        // SAFETY: The indices are checked so they don't return
        //         mutable references to the same item.
        //         The returned values' lifetimes are bound to the
        //         Arena's lifetime via &self, so no further mutations
        //         can occur on the internal data storage.
        unsafe {
            // erase lifetime
            let this = self as *mut Self;

            ((*this).get_mut(a), (*this).get_mut(b))
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner: self.data.iter(),
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}

impl<T> Entry<T> {
    #[inline(always)]
    #[allow(dead_code)]
    fn is_occupied(&self) -> bool {
        matches!(self, Entry::Occupied { .. })
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn is_vacant(&self) -> bool {
        matches!(self, Entry::Vacant { .. })
    }

    fn unwrap_occupied(self) -> (NonZeroUsize, T) {
        if let Entry::Occupied { generation, item } = self {
            (generation, item)
        } else {
            panic!("called `Entry::unwrap_occupied()` on a `Vacant` value")
        }
    }
}

impl Index {
    #[allow(dead_code)]
    pub(crate) fn from_parts(pos: usize, gen: NonZeroUsize) -> Self {
        Index {
            slot: pos,
            generation: gen,
        }
    }
}

// ----------------------------------------------------------------------------
// Iterators

#[derive(Debug)]
pub struct Iter<'a, T> {
    inner: SliceIter<'a, Entry<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.inner.by_ref() {
            match entry {
                Entry::Vacant { .. } => continue,
                Entry::Occupied { item, .. } => return Some(item),
            }
        }

        None
    }
}
