//! Typed dense ids and the tables keyed by them.
//!
//! An [`Id<K>`] is a `u32` index tagged with a marker type `K` (`Id<Region>`, `Id<Good>`), so a
//! region id cannot index a goods table. A [`Table<K, V>`] is a dense `Vec<V>` indexed by
//! `Id<K>`. With `V = K` it is an array of structs; with several tables sharing `K`
//! (`Table<Region, Fixed>` for prices, `Table<Region, i64>` for people) it is one column of a
//! struct of arrays, which is the layout the tick wants. Iteration is always in id order.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Dense typed index into a [`Table`].
pub struct Id<K> {
    index: u32,
    marker: PhantomData<fn() -> K>,
}

impl<K> Id<K> {
    /// From a raw index.
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            marker: PhantomData,
        }
    }

    /// The raw index.
    pub const fn raw(self) -> u32 {
        self.index
    }

    /// The index as `usize`, for slices.
    pub const fn index(self) -> usize {
        self.index as usize
    }
}

impl<K> Clone for Id<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for Id<K> {}

impl<K> PartialEq for Id<K> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<K> Eq for Id<K> {}

impl<K> PartialOrd for Id<K> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for Id<K> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<K> Hash for Id<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<K> fmt::Debug for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = core::any::type_name::<K>();
        let short = name.rsplit("::").next().unwrap_or(name);
        write!(f, "Id<{short}>({})", self.index)
    }
}

impl<K> Serialize for Id<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.index)
    }
}

impl<'de, K> Deserialize<'de> for Id<K> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u32::deserialize(deserializer).map(Self::new)
    }
}

/// Iterator over the ids `0..len` of a table, in order.
pub struct Ids<K> {
    range: Range<u32>,
    marker: PhantomData<fn() -> K>,
}

impl<K> Ids<K> {
    /// The ids `0..len`.
    pub fn new(len: u32) -> Self {
        Self {
            range: 0..len,
            marker: PhantomData,
        }
    }
}

impl<K> Iterator for Ids<K> {
    type Item = Id<K>;

    fn next(&mut self) -> Option<Id<K>> {
        self.range.next().map(Id::new)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<K> DoubleEndedIterator for Ids<K> {
    fn next_back(&mut self) -> Option<Id<K>> {
        self.range.next_back().map(Id::new)
    }
}

impl<K> ExactSizeIterator for Ids<K> {}

/// Dense table of `V` keyed by `Id<K>`. Ids are assigned in push order and never reused.
pub struct Table<K, V = K> {
    values: Vec<V>,
    marker: PhantomData<fn() -> K>,
}

impl<K, V> Table<K, V> {
    /// An empty table.
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            marker: PhantomData,
        }
    }

    /// From values in id order.
    ///
    /// # Panics
    /// When there are more than `u32::MAX` values.
    pub fn from_vec(values: Vec<V>) -> Self {
        assert!(u32::try_from(values.len()).is_ok(), "table larger than u32");
        Self {
            values,
            marker: PhantomData,
        }
    }

    /// `len` values computed from their ids.
    pub fn from_fn(len: u32, f: impl FnMut(Id<K>) -> V) -> Self {
        Self::from_vec(Ids::new(len).map(f).collect())
    }

    /// Appends a value and returns its id.
    ///
    /// # Panics
    /// When the table already holds `u32::MAX` values.
    pub fn push(&mut self, value: V) -> Id<K> {
        let id = Id::new(u32::try_from(self.values.len()).expect("table larger than u32"));
        self.values.push(value);
        id
    }

    /// Number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The value for `id`, if it exists.
    pub fn get(&self, id: Id<K>) -> Option<&V> {
        self.values.get(id.index())
    }

    /// The value for `id`, mutably, if it exists.
    pub fn get_mut(&mut self, id: Id<K>) -> Option<&mut V> {
        self.values.get_mut(id.index())
    }

    /// All ids, in order.
    pub fn ids(&self) -> Ids<K> {
        Ids::new(self.values.len() as u32)
    }

    /// `(id, value)` pairs in id order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Id<K>, &V)> + ExactSizeIterator {
        self.ids().zip(self.values.iter())
    }

    /// `(id, value)` pairs in id order, mutably.
    pub fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = (Id<K>, &mut V)> + ExactSizeIterator {
        self.ids().zip(self.values.iter_mut())
    }

    /// The values in id order. Use it for `rayon` (`par_iter().enumerate()`), whose indexed
    /// `collect` keeps id order.
    pub fn as_slice(&self) -> &[V] {
        &self.values
    }

    /// The values in id order, mutably.
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        &mut self.values
    }

    /// The values, consuming the table.
    pub fn into_vec(self) -> Vec<V> {
        self.values
    }
}

impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V: Clone> Clone for Table<K, V> {
    fn clone(&self) -> Self {
        Self::from_vec(self.values.clone())
    }
}

impl<K, V: PartialEq> PartialEq for Table<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<K, V: Eq> Eq for Table<K, V> {}

impl<K, V: fmt::Debug> fmt::Debug for Table<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Index<Id<K>> for Table<K, V> {
    type Output = V;

    fn index(&self, id: Id<K>) -> &V {
        &self.values[id.index()]
    }
}

impl<K, V> IndexMut<Id<K>> for Table<K, V> {
    fn index_mut(&mut self, id: Id<K>) -> &mut V {
        &mut self.values[id.index()]
    }
}

impl<K, V: Serialize> Serialize for Table<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.values.serialize(serializer)
    }
}

impl<'de, K, V: Deserialize<'de>> Deserialize<'de> for Table<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<V>::deserialize(deserializer)?;
        if u32::try_from(values.len()).is_err() {
            return Err(serde::de::Error::custom("table larger than u32"));
        }
        Ok(Self::from_vec(values))
    }
}

#[cfg(test)]
mod tests {
    use super::{Id, Table};
    use crate::fixed::Fixed;

    struct Region;

    #[test]
    fn ids_are_dense_and_iteration_follows_them() {
        let mut names: Table<Region, &str> = Table::new();
        let a = names.push("Bengal");
        let b = names.push("Sichuan");
        assert_eq!((a.raw(), b.raw()), (0, 1));
        assert_eq!(names[b], "Sichuan");
        assert_eq!(names.get(Id::new(2)), None);
        let order: Vec<_> = names.iter().map(|(id, name)| (id.raw(), *name)).collect();
        assert_eq!(order, [(0, "Bengal"), (1, "Sichuan")]);
        assert_eq!(names.ids().rev().map(Id::raw).collect::<Vec<_>>(), [1, 0]);
        assert_eq!(format!("{a:?}"), "Id<Region>(0)");
    }

    #[test]
    fn columns_share_ids() {
        let prices: Table<Region, Fixed> = Table::from_fn(3, |id| Fixed::from_int(id.raw().into()));
        let mut people: Table<Region, i64> = Table::from_vec(vec![10, 20, 30]);
        for (id, price) in prices.iter() {
            people[id] += price.to_int_trunc();
        }
        assert_eq!(people.as_slice(), &[10, 21, 32]);
        let bytes = postcard::to_allocvec(&people).unwrap();
        assert_eq!(
            postcard::from_bytes::<Table<Region, i64>>(&bytes).unwrap(),
            people
        );
    }
}
