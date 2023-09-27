use std::clone::Clone;
use std::fmt::Debug;
use std::ops::Range;

pub trait Rangeable {
    fn range(&self) -> Range<usize>;
    fn id(&self) -> String;
}

pub trait FunctionTag: Rangeable + Debug + Clone {
    fn transform(&self, text: String) -> String;
}
