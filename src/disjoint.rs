use std::fmt::format;

pub struct Disjointness {
    mask: u64,
}

pub struct Borrowed;

pub unsafe trait Disjoint {
    fn try_borrow(&self, state: &mut u64, mask: u64) -> Result<&mut Self, Borrowed>;
}
