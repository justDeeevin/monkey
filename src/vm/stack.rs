use std::mem::MaybeUninit;

pub struct Stack<T, const SIZE: usize = 2048> {
    data: [MaybeUninit<T>; SIZE],
    top: usize,
}

impl<T: std::fmt::Debug, const SIZE: usize> std::fmt::Debug for Stack<T, SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self.data
                    .iter()
                    .take(self.top)
                    .map(|x| unsafe { x.assume_init_ref() }),
            )
            .finish()
    }
}

impl<T, const SIZE: usize> Default for Stack<T, SIZE> {
    fn default() -> Self {
        Self {
            data: std::array::from_fn(|_| MaybeUninit::uninit()),
            top: 0,
        }
    }
}

impl<T, const SIZE: usize> Stack<T, SIZE> {
    pub fn push(&mut self, value: T) {
        self.data[self.top] = MaybeUninit::new(value);
        self.top += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.top == 0 {
            None
        } else {
            self.top -= 1;
            // SAFETY: We just checked that the top is not out of bounds.
            Some(unsafe { self.data[self.top].assume_init_read() })
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.top == 0 {
            None
        } else {
            // SAFETY: Top is always 1 greater than
            Some(unsafe { self.data[self.top - 1].assume_init_ref() })
        }
    }
}
