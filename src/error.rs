pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Hyper(hyper::Error),
    HttpPare(ParseError),
}