/// A scratch buffer for reusable memory.
///
/// The point of this is to resuse already allocated memory
/// avoiding constant allocations.
pub struct ScratchBuffer<T: 'static> {
    inner: Vec<&'static T>,
}

impl<T> ScratchBuffer<T> {
    pub fn empty() -> Self {
        Self { inner: vec![] }
    }

    /// Create a buffer guard.
    /// SAFETY
    /// The memory is cleared when the guard is dropped.
    pub fn guard<'a>(&'a mut self) -> ScratchGuard<'a, T> {
        let inner = unsafe { std::mem::transmute(&mut self.inner) };
        ScratchGuard { inner }
    }
}

pub struct ScratchGuard<'a, T> {
    inner: &'a mut Vec<&'a T>,
}

impl<'a, T> ScratchGuard<'a, T> {
    pub fn with<F>(self, mut f: F)
    where
        F: FnMut(&mut Vec<&T>),
    {
        f(self.inner);
    }
}

impl<T> Drop for ScratchGuard<'_, T> {
    fn drop(&mut self) {
        self.inner.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Copy, Clone)]
    struct Data<'a>(&'a str);

    struct UseBuffer {
        buffer: ScratchBuffer<Data<'static>>,
        strings: Vec<String>,
    }

    // impl UseBuffer {
    //     fn run(&mut self) {
    //         let guard = self.buffer.guard();

    //         let data = Data(self.strings[0].as_str());
    //         guard.with(|buffer| {
    //             buffer.push(&data);
    //         });
    //     }
    // }

    #[test]
    fn use_guard() {
        panic!()
        // let mut ub = UseBuffer {
        //     buffer: ScratchBuffer::empty(),
        //     strings: vec![String::from("hello")],
        // };

        // ub.run();
    }
}
