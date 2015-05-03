#![feature(convert, alloc, collections)]

#[macro_use] extern crate lazy_static;
extern crate regex;

macro_rules! hashset {
    ( $( $x:expr ),* ) => { {
        let mut temp_hashset = HashSet::new();
        $( temp_hashset.insert($x); )*
        temp_hashset
    } }
}

macro_rules! hashmap {
    ( $( $key:expr => $val:expr ),* ) => { {
        let mut temp_hashmap = HashMap::new();
        $( temp_hashmap.insert($key, $val); )*
        temp_hashmap
    } }
}

macro_rules! btreemap {
    ( $( $key:expr => $val:expr ),* ) => { {
        let mut temp_btreemap = BTreeMap::new();
        $( temp_btreemap.insert($key, $val); )*
        temp_btreemap
    } }
}

pub mod dom;
pub mod util;
