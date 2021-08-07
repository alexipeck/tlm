#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DBTable {
    Content,
    Shows,
    Episode,
}

pub enum Elements {
    Like,
    Equals,
    Not,
}

pub struct Filter {
    elements: Vec<Elements>, //will be a mix of dbtables and elements etc
}

impl Filter {}
