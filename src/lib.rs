#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
extern crate regex;
extern crate uuid;

pub use dom::DOM;

pub mod dom;
pub mod util;
