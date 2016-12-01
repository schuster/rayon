use futures::Future;

struct ScopeFuture<F: Future> {
    future: Option<F>,

    // the counter in the scope; since the scope doesn't terminate until
    // counter reaches zero, and we hold a ref in this counter, we are
    // assured that this pointer remains valid
    counter: *const CountLatch,
}

// Assert that the `*const` is safe to transmit between threads:
unsafe impl<F: Future + Send> Send for ScopeFuture<F> { }
unsafe impl<F: Future + Sync> Sync for ScopeFuture<F> { }

impl<F: Future> ScopeFuture<F> {
    // Unsafe: Caller asserts that `counter` will remain valid until it is decremented.
    unsafe fn new(future: F, counter: *const CountLatch) -> Self {
        ScopeFuture { future: Some(future), counter: counter }
    }
}

impl<F: Future> Drop for ScopeFuture<F> {
    fn drop(&mut self) {
        unsafe {
            // So, this is subtle. We know that the type `F` may have
            // some data which is only valid until the end of the
            // scope, and we also know that the scope doesn't end
            // until `self.counter` is decremented below. So we want
            // to be sure to drop `self.future` first.
            mem::drop(self.future.take());

            // Set the latch. By the struct invariant, we know that
            // the counter pointer will remain valid until this ref is
            // dropped.
            (*self.counter).set();
        }
    }
}

impl<F: Future> Future for ScopeFuture<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.as_mut().unwrap().poll()
    }
}
