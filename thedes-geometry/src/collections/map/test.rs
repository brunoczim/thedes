use crate::{
    collections::map::Entry,
    coords::CoordRange,
    orientation::Axis,
    CoordPair,
};

use super::CoordMap;

#[test]
fn correct_is_empty() {
    let mut map = CoordMap::new();
    assert!(map.is_empty());
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    assert!(!map.is_empty());
}

#[test]
fn basic_insert_correct_length() {
    let mut map = CoordMap::new();
    assert_eq!(map.len(), 0);
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    assert_eq!(map.len(), 1);
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    assert_eq!(map.len(), 2);
    map.insert(CoordPair { y: 13, x: 0 }, "xyzwuv");
    assert_eq!(map.len(), 3);
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
    assert_eq!(map.len(), 3);
    assert_eq!(map.remove(CoordPair { y: 7, x: 12 }.as_ref()), Some("de"));
    assert_eq!(map.len(), 2);
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
    assert_eq!(map.len(), 3);
    assert_eq!(map.remove(CoordPair { y: 6, x: 3 }.as_ref()), None);
    assert_eq!(map.len(), 3);
    assert_eq!(map.remove(CoordPair { y: 5, x: 2 }.as_ref()), None);
    assert_eq!(map.len(), 3);
    assert_eq!(map.remove(CoordPair { y: 6, x: 2 }.as_ref()), None);
    assert_eq!(map.len(), 3);
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
    assert_eq!(map.len(), 3);
    assert_eq!(
        map.remove_entry(CoordPair { y: 13, x: 0 }.as_ref()),
        Some((CoordPair { y: 13, x: 0 }, "xyzwuv"))
    );
    assert_eq!(map.len(), 2);
}

#[test]
fn entry_or_insert() {
    let mut map = CoordMap::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert("ij"), &"ij");
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert("abc"), &"ij");
    assert_eq!(map.len(), 1);
}

#[test]
fn entry_insert_crossing_keys() {
    let mut map = CoordMap::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert("abc"), &"abc");
    assert_eq!(map.entry(CoordPair { y: 0, x: 3 }).or_insert("ij"), &"ij");
    assert_eq!(map.entry(CoordPair { y: 0, x: 2 }).or_insert("pqrs"), &"pqrs");
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert("123"), &"abc");
    assert_eq!(map.entry(CoordPair { y: 0, x: 3 }).or_insert("456"), &"ij");
    assert_eq!(map.entry(CoordPair { y: 0, x: 2 }).or_insert("789"), &"pqrs");
    assert_eq!(map.len(), 3);
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
    assert_eq!(map.len(), 1);
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
    assert_eq!(map.len(), 1);
}

#[test]
fn entry_or_default() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_default(), &false);
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_insert(true), &false);
    assert_eq!(map.len(), 1);
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
    assert_eq!(map.len(), 1);
}

#[test]
fn entry_key() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(
        map.entry(CoordPair { y: 1, x: 2 }).key(),
        CoordPair { y: &1, x: &2 }
    );
    assert_eq!(map.len(), 0);
}

#[test]
fn entry_occupied_insert() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_default(), &false);
    match map.entry(CoordPair { y: 1, x: 2 }) {
        Entry::Vacant(_) => unreachable!(),
        Entry::Occupied(mut entry) => {
            assert_eq!(entry.insert(true), false);
            assert_eq!(entry.get(), &true);
        },
    }
    assert_eq!(map.get(CoordPair { y: &1, x: &2 }), Some(&true));
    assert_eq!(map.len(), 1);
}

#[test]
fn entry_remove() {
    let mut map = CoordMap::<u8, bool>::new();
    assert_eq!(map.entry(CoordPair { y: 1, x: 2 }).or_default(), &false);
    map.entry(CoordPair { y: 0, x: 0 }).or_insert(true);
    match map.entry(CoordPair { y: 1, x: 2 }) {
        Entry::Vacant(_) => unreachable!(),
        Entry::Occupied(entry) => {
            assert_eq!(entry.remove_entry(), (CoordPair { y: 1, x: 2 }, false))
        },
    }
    assert_eq!(map.get(CoordPair { y: &1, x: &2 }), None);
    assert_eq!(map.get(CoordPair { y: &0, x: &0 }), Some(&true));
}

#[test]
fn entry_mixed_with_remove() {
    let mut map = CoordMap::<u8, &str>::new();
    map.entry(CoordPair { y: 3, x: 2 }).or_insert("abc");
    map.entry(CoordPair { y: 0, x: 0 }).or_insert("orig");
    map.entry(CoordPair { y: 16, x: 13 }).or_insert("foobar");
    assert_eq!(map.len(), 3);
    assert_eq!(
        map.remove_entry(CoordPair { y: &3, x: &2 }),
        Some((CoordPair { y: 3, x: 2 }, "abc")),
    );
    assert_eq!(map.len(), 2);
    assert_eq!(
        map.remove_entry(CoordPair { y: &16, x: &13 }),
        Some((CoordPair { y: 16, x: 13 }, "foobar")),
    );
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(CoordPair { y: &3, x: &2 }), None);
    assert_eq!(map.get(CoordPair { y: &16, x: &13 }), None);
    assert_eq!(map.get(CoordPair { y: &0, x: &0 }), Some(&"orig"));
}

#[test]
fn iter_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map.rows().collect();
    assert_eq!(
        &rows[..],
        &[
            (CoordPair { y: &5, x: &1 }, &";:!"),
            (CoordPair { y: &5, x: &3 }, &"abc"),
            (CoordPair { y: &5, x: &9 }, &"123"),
            (CoordPair { y: &5, x: &10 }, &"%#@"),
            (CoordPair { y: &7, x: &12 }, &"de"),
            (CoordPair { y: &13, x: &2 }, &"789456"),
            (CoordPair { y: &13, x: &3 }, &"xyzwuv"),
        ]
    );
}

#[test]
fn iter_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map.columns().collect();
    assert_eq!(
        &columns[..],
        &[
            (CoordPair { y: &5, x: &1 }, &";:!"),
            (CoordPair { y: &13, x: &2 }, &"789456"),
            (CoordPair { y: &5, x: &3 }, &"abc"),
            (CoordPair { y: &13, x: &3 }, &"xyzwuv"),
            (CoordPair { y: &5, x: &9 }, &"123"),
            (CoordPair { y: &5, x: &10 }, &"%#@"),
            (CoordPair { y: &7, x: &12 }, &"de"),
        ]
    );
}

#[test]
fn iter_rows_back_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let mut iter = map.rows();
    assert_eq!(
        iter.next_back(),
        Some((CoordPair { y: &13, x: &3 }, &"xyzwuv"))
    );
    assert_eq!(iter.next(), Some((CoordPair { y: &5, x: &3 }, &"abc")));
    assert_eq!(
        iter.next_back(),
        Some((CoordPair { y: &13, x: &2 }, &"789456"))
    );
    assert_eq!(iter.next_back(), Some((CoordPair { y: &9, x: &4 }, &"xy")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: &5, x: &9 }, &"123")));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_rows_front_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let mut iter = map.rows();
    assert_eq!(iter.next(), Some((CoordPair { y: &5, x: &3 }, &"abc")));
    assert_eq!(
        iter.next_back(),
        Some((CoordPair { y: &13, x: &3 }, &"xyzwuv"))
    );
    assert_eq!(iter.next(), Some((CoordPair { y: &5, x: &9 }, &"123")));
    assert_eq!(iter.next(), Some((CoordPair { y: &9, x: &4 }, &"xy")));
    assert_eq!(iter.next(), Some((CoordPair { y: &13, x: &2 }, &"789456")));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn iter_keys_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map.key_rows().collect();
    assert_eq!(
        &rows[..],
        &[
            CoordPair { y: &5, x: &1 },
            CoordPair { y: &5, x: &3 },
            CoordPair { y: &5, x: &9 },
            CoordPair { y: &5, x: &10 },
            CoordPair { y: &7, x: &12 },
            CoordPair { y: &13, x: &2 },
            CoordPair { y: &13, x: &3 },
        ]
    );
}

#[test]
fn iter_keys_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map.key_columns().collect();
    assert_eq!(
        &columns[..],
        &[
            CoordPair { y: &5, x: &1 },
            CoordPair { y: &13, x: &2 },
            CoordPair { y: &5, x: &3 },
            CoordPair { y: &13, x: &3 },
            CoordPair { y: &5, x: &9 },
            CoordPair { y: &5, x: &10 },
            CoordPair { y: &7, x: &12 },
        ]
    );
}

#[test]
fn iter_values_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map.value_rows().collect();
    assert_eq!(
        &rows[..],
        &[&";:!", &"abc", &"123", &"%#@", &"de", &"789456", &"xyzwuv",]
    );
}

#[test]
fn iter_values_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map.value_columns().collect();
    assert_eq!(
        &columns[..],
        &[&";:!", &"789456", &"abc", &"xyzwuv", &"123", &"%#@", &"de",]
    );
}

#[test]
fn into_iter_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map.into_rows().collect();
    assert_eq!(
        &rows[..],
        &[
            (CoordPair { y: 5, x: 1 }, ";:!"),
            (CoordPair { y: 5, x: 3 }, "abc"),
            (CoordPair { y: 5, x: 9 }, "123"),
            (CoordPair { y: 5, x: 10 }, "%#@"),
            (CoordPair { y: 7, x: 12 }, "de"),
            (CoordPair { y: 13, x: 2 }, "789456"),
            (CoordPair { y: 13, x: 3 }, "xyzwuv"),
        ]
    );
}

#[test]
fn into_iter_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map.into_columns().collect();
    assert_eq!(
        &columns[..],
        &[
            (CoordPair { y: 5, x: 1 }, ";:!"),
            (CoordPair { y: 13, x: 2 }, "789456"),
            (CoordPair { y: 5, x: 3 }, "abc"),
            (CoordPair { y: 13, x: 3 }, "xyzwuv"),
            (CoordPair { y: 5, x: 9 }, "123"),
            (CoordPair { y: 5, x: 10 }, "%#@"),
            (CoordPair { y: 7, x: 12 }, "de"),
        ]
    );
}

#[test]
fn into_iter_rows_back_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let mut iter = map.into_rows();
    assert_eq!(iter.next_back(), Some((CoordPair { y: 13, x: 3 }, "xyzwuv")));
    assert_eq!(iter.next(), Some((CoordPair { y: 5, x: 3 }, "abc")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: 13, x: 2 }, "789456")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: 9, x: 4 }, "xy")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: 5, x: 9 }, "123")));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter_rows_front_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let mut iter = map.into_rows();
    assert_eq!(iter.next(), Some((CoordPair { y: 5, x: 3 }, "abc")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: 13, x: 3 }, "xyzwuv")));
    assert_eq!(iter.next(), Some((CoordPair { y: 5, x: 9 }, "123")));
    assert_eq!(iter.next(), Some((CoordPair { y: 9, x: 4 }, "xy")));
    assert_eq!(iter.next(), Some((CoordPair { y: 13, x: 2 }, "789456")));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn into_iter_keys_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map.into_key_rows().collect();
    assert_eq!(
        rows[..],
        [
            CoordPair { y: 5, x: 1 },
            CoordPair { y: 5, x: 3 },
            CoordPair { y: 5, x: 9 },
            CoordPair { y: 5, x: 10 },
            CoordPair { y: 7, x: 12 },
            CoordPair { y: 13, x: 2 },
            CoordPair { y: 13, x: 3 },
        ]
    );
}

#[test]
fn into_iter_keys_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map.into_key_columns().collect();
    assert_eq!(
        &columns[..],
        &[
            CoordPair { y: 5, x: 1 },
            CoordPair { y: 13, x: 2 },
            CoordPair { y: 5, x: 3 },
            CoordPair { y: 13, x: 3 },
            CoordPair { y: 5, x: 9 },
            CoordPair { y: 5, x: 10 },
            CoordPair { y: 7, x: 12 },
        ]
    );
}

#[test]
fn into_iter_values_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map.into_value_rows().collect();
    assert_eq!(
        &rows[..],
        &[";:!", "abc", "123", "%#@", "de", "789456", "xyzwuv",]
    );
}

#[test]
fn into_iter_values_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 10 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 12 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map.into_value_columns().collect();
    assert_eq!(
        &columns[..],
        &[";:!", "789456", "abc", "xyzwuv", "123", "%#@", "de",]
    );
}

#[test]
fn into_iter_values_rows_back_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let mut iter = map.into_value_rows();
    assert_eq!(iter.next_back(), Some("xyzwuv"));
    assert_eq!(iter.next(), Some("abc"));
    assert_eq!(iter.next_back(), Some("789456"));
    assert_eq!(iter.next_back(), Some("xy"));
    assert_eq!(iter.next_back(), Some("123"));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter_values_rows_front_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let mut iter = map.into_value_rows();
    assert_eq!(iter.next(), Some("abc"));
    assert_eq!(iter.next_back(), Some("xyzwuv"));
    assert_eq!(iter.next(), Some("123"));
    assert_eq!(iter.next(), Some("xy"));
    assert_eq!(iter.next(), Some("789456"));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn range_rows_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let rows: Vec<_> = map
        .range(Axis::Y, &CoordRange { y: 5_u8 .. 8, x: 9_u8 ..= 12 })
        .collect();
    assert_eq!(
        &rows[..],
        &[
            (CoordPair { y: &5, x: &9 }, &"123"),
            (CoordPair { y: &5, x: &12 }, &"%#@"),
            (CoordPair { y: &7, x: &10 }, &"de"),
        ]
    );
}

#[test]
fn range_columns_collect() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    let columns: Vec<_> = map
        .range(Axis::X, &CoordRange { y: 5_u8 .. 8, x: 9_u8 ..= 12 })
        .collect();
    assert_eq!(
        &columns[..],
        &[
            (CoordPair { y: &5, x: &9 }, &"123"),
            (CoordPair { y: &7, x: &10 }, &"de"),
            (CoordPair { y: &5, x: &12 }, &"%#@"),
        ]
    );
}

#[test]
fn range_rows_back_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");
    map.insert(CoordPair { y: 14, x: 0 }, "ijk");

    let mut iter =
        map.range(Axis::Y, &CoordRange { y: 5_u8 ..= 13, x: 3_u8 .. });
    assert_eq!(
        iter.next_back(),
        Some((CoordPair { y: &13, x: &3 }, &"xyzwuv"))
    );
    assert_eq!(iter.next(), Some((CoordPair { y: &5, x: &3 }, &"abc")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: &9, x: &4 }, &"xy")));
    assert_eq!(iter.next_back(), Some((CoordPair { y: &5, x: &9 }, &"123")));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn range_rows_front_wraps() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 9, x: 4 }, "xy");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");
    map.insert(CoordPair { y: 14, x: 0 }, "ijk");

    let mut iter = map.range(Axis::Y, &CoordRange { y: 5_u8 ..= 13, x: .. });
    assert_eq!(iter.next(), Some((CoordPair { y: &5, x: &3 }, &"abc")));
    assert_eq!(
        iter.next_back(),
        Some((CoordPair { y: &13, x: &3 }, &"xyzwuv"))
    );
    assert_eq!(iter.next(), Some((CoordPair { y: &5, x: &9 }, &"123")));
    assert_eq!(iter.next(), Some((CoordPair { y: &9, x: &4 }, &"xy")));
    assert_eq!(iter.next(), Some((CoordPair { y: &13, x: &2 }, &"789456")));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn next_neighbor_finds() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 8 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.next_neighbor(Axis::X, CoordPair { y: 7, x: 8 }.as_ref()),
        Some((CoordPair { y: &7, x: &10 }, &"de")),
    );

    assert_eq!(
        map.next_neighbor(Axis::Y, CoordPair { y: 5, x: 3 }.as_ref()),
        Some((CoordPair { y: &13, x: &3 }, &"xyzwuv")),
    );
}

#[test]
fn next_neighbor_does_not_find() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 8 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.next_neighbor(Axis::X, CoordPair { y: 7, x: 10 }.as_ref()),
        None,
    );

    assert_eq!(
        map.next_neighbor(Axis::Y, CoordPair { y: 7, x: 10 }.as_ref()),
        None,
    );
}

#[test]
fn prev_neighbor_finds() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 8 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.prev_neighbor(Axis::X, CoordPair { y: 7, x: 10 }.as_ref()),
        Some((CoordPair { y: &7, x: &8 }, &"blergh")),
    );

    assert_eq!(
        map.prev_neighbor(Axis::Y, CoordPair { y: 13, x: 3 }.as_ref()),
        Some((CoordPair { y: &5, x: &3 }, &"abc")),
    );
}

#[test]
fn prev_neighbor_does_not_find() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 8 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.prev_neighbor(Axis::X, CoordPair { y: 7, x: 8 }.as_ref()),
        None,
    );

    assert_eq!(
        map.prev_neighbor(Axis::Y, CoordPair { y: 7, x: 10 }.as_ref()),
        None,
    );
}

#[test]
fn last_neighbor_finds() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 3 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.last_neighbor(Axis::X, CoordPair { y: 5, x: 3 }.as_ref()),
        Some((CoordPair { y: &5, x: &12 }, &"%#@")),
    );

    assert_eq!(
        map.last_neighbor(Axis::Y, CoordPair { y: 5, x: 3 }.as_ref()),
        Some((CoordPair { y: &13, x: &3 }, &"xyzwuv")),
    );
}

#[test]
fn last_neighbor_does_not_find() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 8 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.last_neighbor(Axis::X, CoordPair { y: 7, x: 10 }.as_ref()),
        None,
    );

    assert_eq!(
        map.last_neighbor(Axis::Y, CoordPair { y: 7, x: 10 }.as_ref()),
        None,
    );
}

#[test]
fn first_neighbor_finds() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 3 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.first_neighbor(Axis::X, CoordPair { y: 5, x: 12 }.as_ref()),
        Some((CoordPair { y: &5, x: &1 }, &";:!")),
    );

    assert_eq!(
        map.first_neighbor(Axis::Y, CoordPair { y: 13, x: 3 }.as_ref()),
        Some((CoordPair { y: &5, x: &3 }, &"abc")),
    );
}

#[test]
fn first_neighbor_does_not_find() {
    let mut map = CoordMap::<u8, &str>::new();
    map.insert(CoordPair { y: 5, x: 3 }, "abc");
    map.insert(CoordPair { y: 5, x: 9 }, "123");
    map.insert(CoordPair { y: 5, x: 12 }, "%#@");
    map.insert(CoordPair { y: 5, x: 1 }, ";:!");
    map.insert(CoordPair { y: 7, x: 10 }, "de");
    map.insert(CoordPair { y: 7, x: 8 }, "blergh");
    map.insert(CoordPair { y: 13, x: 3 }, "xyzwuv");
    map.insert(CoordPair { y: 13, x: 2 }, "789456");

    assert_eq!(
        map.first_neighbor(Axis::X, CoordPair { y: 7, x: 0 }.as_ref()),
        None,
    );

    assert_eq!(
        map.first_neighbor(Axis::Y, CoordPair { y: 7, x: 10 }.as_ref()),
        None,
    );
}
