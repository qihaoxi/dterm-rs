use std::error::Error;

pub struct MyError {
    inner: Box<dyn std::error::Error + Send>,
}

impl MyError {
    pub(crate) fn new(p0: Box<dyn Error+Send>) -> Self {
        Self {
            inner: p0,
        }
    }
}

impl std::fmt::Debug for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl TryFrom<Box<dyn std::error::Error + Send>> for MyError {
    type Error = Box<dyn std::error::Error + Send>;

    fn try_from(p0: Box<dyn std::error::Error + Send>) -> Result<MyError, Self::Error> {
        Ok(MyError::new(p0))
    }
}

impl std::error::Error for MyError {}