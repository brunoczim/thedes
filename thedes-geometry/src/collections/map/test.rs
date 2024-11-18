use crate::{collections::map::Entry, CoordPair};

use super::CoordMap;

#[test]
fn correct_is_empty() {
    let mut map = CoordMap::new();
    assert!(map.is_empty());
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    assert!(!map.is_empty());
}

#[test]
fn get_unknown() {
    let mut map = CoordMap::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 0 }, "xyzwuv");
    assert_eq!(map.get(CoordPair { y: 6, x: 3 }.as_ref()), None);
    assert_eq!(map.get(CoordPair { y: 5, x: 2 }.as_ref()), None);
    assert_eq!(map.get(CoordPair { y: 6, x: 2 }.as_ref()), None);
}

#[test]
fn get_known() {
    let mut map = CoordMap::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 0 }, "xyzwuv");
    assert_eq!(map.get(CoordPair { y: 5, x: 3 }.as_ref()), Some(&"abc"));
    assert_eq!(map.get(CoordPair { y: 7, x: 12 }.as_ref()), Some(&"de"));
    assert_eq!(map.get(CoordPair { y: 13, x: 0 }.as_ref()), Some(&"xyzwuv"));
}

#[test]
fn remove_success() {
    let mut map = CoordMap::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 0 }, "xyzwuv");
    assert_eq!(map.remove(CoordPair { y: 7, x: 12 }.as_ref()), Some("de"));
    assert_eq!(map.get(CoordPair { y: 5, x: 3 }.as_ref()), Some(&"abc"));
    assert_eq!(map.get(CoordPair { y: 7, x: 12 }.as_ref()), None);
    assert_eq!(map.get(CoordPair { y: 13, x: 0 }.as_ref()), Some(&"xyzwuv"));
}

#[test]
fn remove_fail() {
    let mut map = CoordMap::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 0 }, "xyzwuv");
    assert_eq!(map.remove(CoordPair { y: 6, x: 3 }.as_ref()), None);
    assert_eq!(map.remove(CoordPair { y: 5, x: 2 }.as_ref()), None);
    assert_eq!(map.remove(CoordPair { y: 6, x: 2 }.as_ref()), None);
    assert_eq!(map.get(CoordPair { y: 5, x: 3 }.as_ref()), Some(&"abc"));
    assert_eq!(map.get(CoordPair { y: 7, x: 12 }.as_ref()), Some(&"de"));
    assert_eq!(map.get(CoordPair { y: 13, x: 0 }.as_ref()), Some(&"xyzwuv"));
}

#[test]
fn remove_entry_success() {
    let mut map = CoordMap::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 0 }, "xyzwuv");
    assert_eq!(
        map.remove_entry(CoordPair { y: 13, x: 0 }.as_ref()),
        Some((CoordPair { y: 13, x: 0 }, "xyzwuv"))
    );
}

#[test]
fn entry_or_insert() {
    let mut map = CoordMap::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert("ij"), &"ij");
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert("abc"), &"ij");
}

#[test]
fn entry_or_insert_with() {
    let mut map = CoordMap::new();
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 }).or_insert_with(|| "ij"),
        &"ij"
    );
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 }).or_insert_with(|| "abc"),
        &"ij"
    );
}

#[test]
fn entry_or_insert_with_key() {
    let mut map = CoordMap::new();
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 })
            .or_insert_with_key(|key| key.y + key.x),
        &3
    );
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 }).or_insert_with_key(|_| 0),
        &3
    );
}

#[test]
fn entry_or_default() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_default(), &false);
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert(true), &false);
}

#[test]
fn entry_and_modify() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 })
            .and_modify(|v| *v = true)
            .or_default(),
        &false
    );
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 })
            .and_modify(|v| *v = true)
            .or_default(),
        &true
    );
}

#[test]
fn entry_key() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 }).key(),
        CoordPair { y: &1, x: &2 }
    );
}

#[test]
fn entry_remove() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_default(), &false);
    match map.entry(CoordPair { y: 1, x: 2 }) {
        Entry::Vacant(_) => unreachable!(),
        Entry::Occupied(entry) => {
            assert_eq!(entry.remove_entry(), (CoordPair { y: 1, x: 2 }, false))
        },
    }
}

#[test]
fn entry_mixed_with_remove() {
    let mut map = CoordMap::<u8, &str>::new();
    map.entry(CoordPair { y: 3, x: 2 }).or_insert("abc");
    map.entry(CoordPair { y: 0, x: 0 }).or_insert("orig");
    map.entry(CoordPair { y: 16, x: 13 }).or_insert("foobar");
    assert_eq!(
        map.remove_entry(CoordPair { y: &3, x: &2 }),
        Some((CoordPair { y: 3, x: 2 }, "abc")),
    );
    assert_eq!(
        map.remove_entry(CoordPair { y: &16, x: &13 }),
        Some((CoordPair { y: 16, x: 13 }, "foobar")),
    );
    assert_eq!(map.get(CoordPair { y: &3, x: &&2 }), None);
    assert_eq!(map.get(CoordPair { y: &16, x: &13 }), None);
    assert_eq!(map.get(CoordPair { y: &0, x: &0 }), Some(&"orig"));
}
