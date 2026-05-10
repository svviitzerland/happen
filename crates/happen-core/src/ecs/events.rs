pub trait Event: Send + Sync + 'static {}

pub struct Events<E: Event> {
    buffer_a: Vec<E>,
    buffer_b: Vec<E>,
    use_a: bool,
}

impl<E: Event> Events<E> {
    pub fn new() -> Self {
        Self {
            buffer_a: Vec::new(),
            buffer_b: Vec::new(),
            use_a: true,
        }
    }

    pub fn send(&mut self, event: E) {
        if self.use_a {
            self.buffer_a.push(event);
        } else {
            self.buffer_b.push(event);
        }
    }

    pub fn drain(&mut self) -> impl Iterator<Item = E> + '_ {
        if self.use_a {
            self.buffer_b.drain(..)
        } else {
            self.buffer_a.drain(..)
        }
    }

    pub fn read(&self) -> &[E] {
        if self.use_a {
            &self.buffer_b
        } else {
            &self.buffer_a
        }
    }

    pub fn swap(&mut self) {
        self.use_a = !self.use_a;
    }

    pub fn clear(&mut self) {
        self.buffer_a.clear();
        self.buffer_b.clear();
    }
}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self::new()
    }
}

