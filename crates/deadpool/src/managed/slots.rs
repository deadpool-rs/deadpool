//! Slot and permit bookkeeping of the managed [`Pool`].
//!
//! A pool of `max_size` has `max_size` slots. Every slot is in one of
//! three states:
//!
//! - **available**: represented by a free permit of the semaphore. An
//!   available slot either holds an idle object or is empty.
//! - **acquired**: represented by a [`Permit`]. The holder either took an
//!   idle object out of the pool via [`Permit::pop`] or creates a new one
//!   and registers it via [`Permit::occupy`].
//! - **leased**: the permit was turned into an object which is borrowed
//!   by the user via [`Permit::lease`]. It's handed back via
//!   [`Slots::release`] or [`Slots::remove`].
//!
//! When the pool is shrunk the semaphore permits of acquired and leased
//! slots can't be taken away immediately. They're recorded as `owed`
//! and forgotten once they're handed back. This keeps the following
//! invariant (with `held` being the number of acquired and leased slots):
//!
//! ```text
//! available_permits + held == max_size + owed
//! ```
//!
//! [`Pool`]: super::Pool

use std::{
    collections::VecDeque,
    sync::{Mutex, MutexGuard},
};

use tokio::sync::{AcquireError, Semaphore, TryAcquireError};

use super::QueueMode;

#[derive(Debug)]
pub(crate) struct Slots<T> {
    state: Mutex<State<T>>,
    semaphore: Semaphore,
}

#[derive(Debug)]
struct State<T> {
    /// Idle objects
    idle: VecDeque<T>,
    /// Number of objects which belong to the pool: idle objects plus the
    /// ones held by acquired and leased slots.
    size: usize,
    max_size: usize,
    /// Number of permits which still need to be removed from the semaphore
    /// due to the pool being shrunk while slots were acquired or leased.
    owed: usize,
}

/// Size information of [`Slots`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SlotsStatus {
    pub(crate) size: usize,
    pub(crate) max_size: usize,
}

impl<T> Slots<T> {
    pub(crate) fn new(max_size: usize) -> Self {
        Self {
            state: Mutex::new(State {
                idle: VecDeque::with_capacity(max_size),
                size: 0,
                max_size,
                owed: 0,
            }),
            semaphore: Semaphore::new(max_size),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State<T>> {
        self.state.lock().unwrap()
    }

    /// Waits for a slot to become available. This is fair: slots are
    /// handed out in the order they were requested.
    pub(crate) async fn acquire(&self) -> Result<Permit<'_, T>, AcquireError> {
        self.semaphore.acquire().await.map(|p| self.permit(p))
    }

    pub(crate) fn try_acquire(&self) -> Result<Permit<'_, T>, TryAcquireError> {
        self.semaphore.try_acquire().map(|p| self.permit(p))
    }

    fn permit(&self, permit: tokio::sync::SemaphorePermit<'_>) -> Permit<'_, T> {
        // The semaphore permit is returned manually by `return_permit`
        // which takes care of `owed` permits.
        permit.forget();
        Permit {
            slots: self,
            occupied: false,
        }
    }

    /// Returns a permit of an acquired or leased slot to the semaphore
    /// unless it's owed due to the pool being shrunk.
    fn return_permit(&self, state: &mut State<T>) {
        if state.owed > 0 {
            state.owed -= 1;
        } else {
            self.semaphore.add_permits(1);
        }
    }

    /// Hands back the object of a leased slot.
    ///
    /// Returns the object if it can't be put back into the pool because
    /// the pool was shrunk. The caller is responsible for disposing of it.
    #[must_use]
    pub(crate) fn release(&self, obj: T) -> Option<T> {
        let mut state = self.lock();
        let rejected = if state.owed > 0 {
            state.size -= 1;
            Some(obj)
        } else {
            debug_assert!(state.size <= state.max_size);
            state.idle.push_back(obj);
            None
        };
        self.return_permit(&mut state);
        rejected
    }

    /// Hands back a leased slot whose object is removed from the pool.
    pub(crate) fn remove(&self) {
        let mut state = self.lock();
        state.size -= 1;
        self.return_permit(&mut state);
    }

    /// Changes the `max_size`.
    ///
    /// Returns the idle objects which were removed from the pool.
    #[must_use]
    pub(crate) fn resize(&self, max_size: usize) -> Vec<T> {
        let mut removed = Vec::new();
        if self.semaphore.is_closed() {
            return removed;
        }
        let mut state = self.lock();
        let old_max_size = state.max_size;
        state.max_size = max_size;
        if max_size < old_max_size {
            let diff = old_max_size - max_size;
            state.owed += diff - self.semaphore.forget_permits(diff);
            while state.size > state.max_size {
                let Some(obj) = state.idle.pop_front() else {
                    break;
                };
                state.size -= 1;
                removed.push(obj);
            }
            state.idle.shrink_to(max_size);
        } else if max_size > old_max_size {
            let diff = max_size - old_max_size;
            let paid = diff.min(state.owed);
            state.owed -= paid;
            state.idle.reserve_exact(diff);
            self.semaphore.add_permits(diff - paid);
        }
        removed
    }

    /// Retains only the idle objects specified by the given predicate.
    ///
    /// Returns the number of retained objects and the removed ones.
    pub(crate) fn retain(&self, mut predicate: impl FnMut(&mut T) -> bool) -> (usize, Vec<T>) {
        let mut state = self.lock();
        let mut removed = Vec::new();
        let mut i = 0;
        // This code can be simplified once `VecDeque::extract_if` lands in stable Rust.
        while i < state.idle.len() {
            if predicate(&mut state.idle[i]) {
                i += 1;
            } else {
                removed.push(state.idle.remove(i).unwrap());
            }
        }
        state.size -= removed.len();
        (i, removed)
    }

    /// Resizes to 0 and makes all current and future calls to
    /// [`Slots::acquire`] fail.
    ///
    /// Returns the idle objects which were removed from the pool.
    #[must_use]
    pub(crate) fn close(&self) -> Vec<T> {
        let removed = self.resize(0);
        self.semaphore.close();
        removed
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.semaphore.is_closed()
    }

    pub(crate) fn status(&self) -> SlotsStatus {
        let state = self.lock();
        SlotsStatus {
            size: state.size,
            max_size: state.max_size,
        }
    }
}

/// An acquired slot.
///
/// Dropping it hands the slot back. If it's [occupied](Permit::occupy)
/// at that time the object is considered discarded.
#[must_use]
#[derive(Debug)]
pub(crate) struct Permit<'a, T> {
    slots: &'a Slots<T>,
    /// Whether an object is counted in `size` for this slot.
    occupied: bool,
}

impl<T> Permit<'_, T> {
    /// Takes an idle object out of the pool. On success this permit is
    /// occupied by that object.
    pub(crate) fn pop(&mut self, queue_mode: QueueMode) -> Option<T> {
        debug_assert!(!self.occupied);
        let mut state = self.slots.lock();
        let obj = match queue_mode {
            QueueMode::Fifo => state.idle.pop_front(),
            QueueMode::Lifo => state.idle.pop_back(),
        };
        self.occupied = obj.is_some();
        obj
    }

    /// Registers a newly created object for this slot.
    ///
    /// If the pool was shrunk while the object was being created the pool
    /// might already be full. In that case an idle object is evicted to make
    /// room for the new one and returned. The caller is responsible for
    /// disposing of it.
    #[must_use]
    pub(crate) fn occupy(&mut self) -> Option<T> {
        debug_assert!(!self.occupied);
        let mut state = self.slots.lock();
        self.occupied = true;
        if state.size >= state.max_size {
            if let Some(obj) = state.idle.pop_front() {
                return Some(obj);
            }
        }
        state.size += 1;
        None
    }

    /// Discards the object of this slot, e.g. due to a failed recycle.
    /// The slot can be reused for another object afterwards.
    pub(crate) fn vacate(&mut self) {
        debug_assert!(self.occupied);
        self.slots.lock().size -= 1;
        self.occupied = false;
    }

    /// Turns this permit into a leased slot. The caller is responsible for
    /// handing it back via [`Slots::release`] or [`Slots::remove`].
    pub(crate) fn lease(self) {
        debug_assert!(self.occupied);
        std::mem::forget(self);
    }
}

impl<T> Drop for Permit<'_, T> {
    fn drop(&mut self) {
        let mut state = self.slots.lock();
        if self.occupied {
            state.size -= 1;
        }
        self.slots.return_permit(&mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available<T>(slots: &Slots<T>) -> usize {
        slots.semaphore.available_permits()
    }

    fn owed<T>(slots: &Slots<T>) -> usize {
        slots.lock().owed
    }

    fn idle<T>(slots: &Slots<T>) -> usize {
        slots.lock().idle.len()
    }

    /// Acquires a slot, creates an object and leases it.
    fn lease(slots: &Slots<u32>, obj: u32) -> u32 {
        let mut permit = slots.try_acquire().unwrap();
        assert_eq!(permit.pop(QueueMode::Fifo), None);
        assert_eq!(permit.occupy(), None);
        permit.lease();
        obj
    }

    #[test]
    fn new() {
        let slots = Slots::<u32>::new(3);
        assert_eq!(available(&slots), 3);
        assert_eq!(
            slots.status(),
            SlotsStatus {
                size: 0,
                max_size: 3
            }
        );
    }

    #[test]
    fn lease_and_release() {
        let slots = Slots::new(2);
        let obj = lease(&slots, 1);
        assert_eq!(available(&slots), 1);
        assert_eq!(slots.status().size, 1);
        assert_eq!(slots.release(obj), None);
        assert_eq!(available(&slots), 2);
        assert_eq!(idle(&slots), 1);
        assert_eq!(slots.status().size, 1);
    }

    #[test]
    fn pop_queue_mode() {
        let slots = Slots::new(2);
        let (a, b) = (lease(&slots, 1), lease(&slots, 2));
        assert_eq!(slots.release(a), None);
        assert_eq!(slots.release(b), None);
        let mut permit = slots.try_acquire().unwrap();
        assert_eq!(permit.pop(QueueMode::Lifo), Some(2));
        drop(permit);
        let mut permit = slots.try_acquire().unwrap();
        assert_eq!(permit.pop(QueueMode::Fifo), Some(1));
    }

    #[test]
    fn drop_permit() {
        let slots = Slots::<u32>::new(1);
        drop(slots.try_acquire().unwrap());
        assert_eq!(available(&slots), 1);
        let mut permit = slots.try_acquire().unwrap();
        assert_eq!(permit.occupy(), None);
        assert_eq!(slots.status().size, 1);
        drop(permit);
        assert_eq!(slots.status().size, 0);
        assert_eq!(available(&slots), 1);
    }

    #[test]
    fn vacate() {
        let slots = Slots::new(1);
        let obj = lease(&slots, 1);
        assert_eq!(slots.release(obj), None);
        let mut permit = slots.try_acquire().unwrap();
        assert_eq!(permit.pop(QueueMode::Fifo), Some(1));
        permit.vacate();
        assert_eq!(slots.status().size, 0);
        assert_eq!(permit.occupy(), None);
        permit.lease();
        assert_eq!(slots.status().size, 1);
        assert_eq!(available(&slots), 0);
    }

    #[test]
    fn remove() {
        let slots = Slots::new(1);
        let _obj = lease(&slots, 1);
        slots.remove();
        assert_eq!(slots.status().size, 0);
        assert_eq!(available(&slots), 1);
    }

    #[test]
    fn shrink_empty() {
        let slots = Slots::<u32>::new(5);
        assert!(slots.resize(0).is_empty());
        assert_eq!(available(&slots), 0);
        assert_eq!(owed(&slots), 0);
    }

    #[test]
    fn shrink_idle() {
        let slots = Slots::new(5);
        let (a, b) = (lease(&slots, 1), lease(&slots, 2));
        assert_eq!(slots.release(a), None);
        assert_eq!(slots.release(b), None);
        assert_eq!(slots.resize(1), vec![1]);
        assert_eq!(
            slots.status(),
            SlotsStatus {
                size: 1,
                max_size: 1
            }
        );
        assert_eq!(available(&slots), 1);
        assert_eq!(owed(&slots), 0);
    }

    #[test]
    fn shrink_leased() {
        let slots = Slots::new(5);
        let (a, b) = (lease(&slots, 1), lease(&slots, 2));
        assert!(slots.resize(1).is_empty());
        assert_eq!(available(&slots), 0);
        assert_eq!(owed(&slots), 1);
        assert_eq!(slots.release(a), Some(1));
        assert_eq!(owed(&slots), 0);
        assert_eq!(available(&slots), 0);
        assert_eq!(slots.release(b), None);
        assert_eq!(available(&slots), 1);
        assert_eq!(
            slots.status(),
            SlotsStatus {
                size: 1,
                max_size: 1
            }
        );
    }

    // A `Pool::get` holding a permit while its object creation fails.
    // https://github.com/deadpool-rs/deadpool/issues/504
    #[test]
    fn shrink_acquired() {
        let slots = Slots::<u32>::new(2);
        let permit = slots.try_acquire().unwrap();
        assert!(slots.resize(0).is_empty());
        assert_eq!(owed(&slots), 1);
        drop(permit);
        assert_eq!(owed(&slots), 0);
        assert_eq!(available(&slots), 0);
    }

    // An object is returned and the pool shrunk while another one is
    // being created.
    #[test]
    fn shrink_creating() {
        let slots = Slots::new(2);
        let mut creating = slots.try_acquire().unwrap();
        assert_eq!(creating.pop(QueueMode::Fifo), None);
        let obj = lease(&slots, 1);
        assert_eq!(slots.release(obj), None);
        assert!(slots.resize(1).is_empty());
        assert_eq!(available(&slots), 0);
        assert_eq!(creating.occupy(), Some(1));
        creating.lease();
        assert_eq!(
            slots.status(),
            SlotsStatus {
                size: 1,
                max_size: 1
            }
        );
        assert_eq!(slots.release(2), None);
        assert_eq!(available(&slots), 1);
    }

    #[test]
    fn shrink_grow_leased() {
        let slots = Slots::new(2);
        let (a, b) = (lease(&slots, 1), lease(&slots, 2));
        assert!(slots.resize(0).is_empty());
        assert_eq!(owed(&slots), 2);
        assert!(slots.resize(1).is_empty());
        assert_eq!(owed(&slots), 1);
        assert_eq!(available(&slots), 0);
        assert_eq!(slots.release(a), Some(1));
        assert_eq!(slots.release(b), None);
        assert_eq!(available(&slots), 1);
        assert_eq!(
            slots.status(),
            SlotsStatus {
                size: 1,
                max_size: 1
            }
        );
    }

    #[test]
    fn retain() {
        let slots = Slots::new(3);
        let objs = [lease(&slots, 1), lease(&slots, 2), lease(&slots, 3)];
        for obj in objs {
            assert_eq!(slots.release(obj), None);
        }
        assert_eq!(slots.retain(|obj| *obj != 2), (2, vec![2]));
        assert_eq!(slots.status().size, 2);
        assert_eq!(available(&slots), 3);
    }

    #[test]
    fn close() {
        let slots = Slots::new(2);
        let (a, b) = (lease(&slots, 1), lease(&slots, 2));
        assert_eq!(slots.release(a), None);
        assert_eq!(slots.close(), vec![1]);
        assert!(slots.is_closed());
        assert!(matches!(slots.try_acquire(), Err(TryAcquireError::Closed)));
        assert_eq!(slots.release(b), Some(2));
        assert_eq!(
            slots.status(),
            SlotsStatus {
                size: 0,
                max_size: 0
            }
        );
        assert!(slots.resize(5).is_empty());
        assert_eq!(slots.status().max_size, 0);
    }

    #[tokio::test]
    async fn acquire_waits_for_release() {
        let slots = Slots::new(1);
        let obj = lease(&slots, 1);
        let mut acquire = std::pin::pin!(slots.acquire());
        assert!(poll_once(acquire.as_mut()).is_none());
        assert_eq!(slots.release(obj), None);
        let mut permit = acquire.await.unwrap();
        assert_eq!(permit.pop(QueueMode::Fifo), Some(1));
    }

    #[tokio::test]
    async fn acquire_fails_on_close() {
        let slots = Slots::<u32>::new(0);
        let mut acquire = std::pin::pin!(slots.acquire());
        assert!(poll_once(acquire.as_mut()).is_none());
        drop(slots.close());
        assert!(acquire.await.is_err());
    }

    fn poll_once<F: Future>(f: std::pin::Pin<&mut F>) -> Option<F::Output> {
        use std::task::{Context, Poll, Waker};
        match f.poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(v) => Some(v),
            Poll::Pending => None,
        }
    }

    /// Minimal xorshift PRNG so the model test needs no extra dependency.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self, n: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % n as u64) as usize
        }
    }

    #[derive(Debug, PartialEq)]
    enum Held {
        /// Just acquired
        Vacant,
        /// No idle object was found and a new one is being created
        Creating,
        /// Holding an object
        Occupied(u32),
    }

    /// Applies random sequences of operations and checks the invariants
    /// of [`Slots`] after each of them.
    #[test]
    fn model() {
        for seed in 1..=2000u64 {
            let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
            let slots = Slots::<u32>::new(rng.next(5));
            let mut held: Vec<(Permit<'_, u32>, Held)> = Vec::new();
            let mut leased: Vec<u32> = Vec::new();
            let mut next_id = 0;
            for step in 0..200 {
                match rng.next(10) {
                    0 | 1 => {
                        if let Ok(permit) = slots.try_acquire() {
                            held.push((permit, Held::Vacant));
                        }
                    }
                    2 if !held.is_empty() => {
                        let i = rng.next(held.len());
                        let (permit, state) = &mut held[i];
                        match state {
                            Held::Vacant => match permit.pop(QueueMode::Fifo) {
                                Some(obj) => *state = Held::Occupied(obj),
                                None => *state = Held::Creating,
                            },
                            Held::Creating => {
                                let _ = permit.occupy();
                                next_id += 1;
                                *state = Held::Occupied(next_id);
                            }
                            Held::Occupied(_) => {
                                permit.vacate();
                                *state = Held::Vacant;
                            }
                        }
                    }
                    3 if !held.is_empty() => {
                        drop(held.swap_remove(rng.next(held.len())));
                    }
                    4 | 5 if !held.is_empty() => {
                        let i = rng.next(held.len());
                        if let Held::Occupied(obj) = held[i].1 {
                            held.swap_remove(i).0.lease();
                            leased.push(obj);
                        }
                    }
                    6 if !leased.is_empty() => {
                        let obj = leased.swap_remove(rng.next(leased.len()));
                        if rng.next(4) == 0 {
                            slots.remove();
                        } else {
                            let _ = slots.release(obj);
                        }
                    }
                    7 => {
                        let _ = slots.resize(rng.next(6));
                    }
                    8 => {
                        let _ = slots.retain(|obj| *obj % 3 != 0);
                    }
                    _ => {}
                }
                let ctx = format!("seed {seed}, step {step}");
                check(&slots, &held, &leased, &ctx);
            }
            // Hand everything back. The pool must end up in a clean state.
            drop(held);
            for obj in leased {
                let _ = slots.release(obj);
            }
            let state = slots.lock();
            assert_eq!(state.owed, 0, "seed {seed}");
            assert_eq!(available(&slots), state.max_size, "seed {seed}");
            assert_eq!(state.idle.len(), state.size, "seed {seed}");
            assert!(state.size <= state.max_size, "seed {seed}");
        }
    }

    fn check(slots: &Slots<u32>, held: &[(Permit<'_, u32>, Held)], leased: &[u32], ctx: &str) {
        let state = slots.lock();
        let available = available(slots);
        let held_count = held.len() + leased.len();
        assert_eq!(
            available + held_count,
            state.max_size + state.owed,
            "permits ({ctx})"
        );
        assert!(
            state.owed == 0 || available == 0,
            "owed with available permits ({ctx})"
        );
        let occupied = held
            .iter()
            .filter(|(_, h)| matches!(h, Held::Occupied(_)))
            .count();
        assert_eq!(
            state.size,
            state.idle.len() + occupied + leased.len(),
            "size ({ctx})"
        );
        assert!(
            state.size <= state.max_size + state.owed,
            "size {} exceeds max_size {} + owed {} ({ctx})",
            state.size,
            state.max_size,
            state.owed,
        );
    }
}
