pub struct TakeOnce<T>(Option<T>);

impl<T> TakeOnce<T> {
    pub fn new(val: T) -> Self {
        Self(Some(val))
    }

    pub fn from_option(val: Option<T>) -> Self {
        Self(val)
    }

    pub fn take(&mut self) -> Option<T> {
        let ret = self.0.take();
        self.0 = None;
        ret
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.as_ref()
    }
}
