pub struct TakeOnce<T, E>(Option<T>, Option<E>);

impl<T, E> TakeOnce<T, E> {
    pub fn new(val: T) -> Self {
        Self(Some(val), None)
    }

    pub fn from_option(val: Option<T>) -> Self {
        Self(val, None)
    }

    pub fn take(&mut self) -> Option<T> {
        let ret = self.0.take();
        self.0 = None;
        ret
    }

    pub fn reply(&mut self, e: E) -> Option<T> {
        let t = self.take()?;
        self.1 = Some(e);
        Some(t)
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.as_ref()
    }

    pub fn response(&self) -> Option<&E> {
        self.1.as_ref()
    }
}
