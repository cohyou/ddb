#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NoPage,
    NotFound,
    FullLeaf,
}