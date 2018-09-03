//! This submodule provides you with the types you will be willing to use
//! as basic types to implement the variables of your CP model.
//! Namely, this submodule provides the following types:
//!   - ReversibleInt (an integer (isize) whose value can be easily reset.

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use ::context::{Trail, TrailEntry};

/// This is the reversible int abstraction. It holds a reference to its
/// parent context. This way, it will be able to post entries on the trail.
pub struct ReversibleInt {
    trail: Rc<RefCell<Trail>>,
    clock: usize,
    value: isize
}

impl ReversibleInt {
    /// Creates a new reversible int associated with the given trail and
    /// initialized with the given value.
    pub fn new(trail: Rc<RefCell<Trail>>, initial: isize) -> ReversibleInt {
        let clock = trail.borrow().clock();
        ReversibleInt {
            trail,
            clock,
            value: initial
        }
    }

    /// This private method takes care of posting an entry on the trail
    /// so as to easily restore the current state.
    fn trail(&mut self) {
        let trail_time = self.trail.borrow().clock();

        if trail_time != self.clock {
            self.clock = trail_time;

            let val = self.value;
            let me = self as (*mut Self);
            let entry = ResetValueTrailEntry::new(me, val);
            self.trail.borrow_mut().push_on_trail(Box::new(entry));
        }
    }

    /// Returns the current value of the reversible integer
    pub fn get_value(&self) -> isize {
        self.value
    }

    /// Changes the current value of the reversible integer
    pub fn set_value(&mut self, v: isize) {
        if v != self.value {
            self.trail();
            self.value = v;
        }
        self.value = v
    }

}

impl fmt::Display for ReversibleInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reversible({})", self.value)
    }
}


/// This structure basically plays the role of a stateful capturing closure.
/// Its purpose is to be able to reset the value of some reversible integer
/// upon `trail pop`
pub struct ResetValueTrailEntry {
    target: *mut ReversibleInt,
    value : isize
}

impl ResetValueTrailEntry {
    pub fn new(me: *mut ReversibleInt, value: isize) -> ResetValueTrailEntry {
        ResetValueTrailEntry {
            target: me,
            value : value
        }
    }
}
impl TrailEntry for ResetValueTrailEntry {
    fn restore(&mut self) {
        unsafe {(*self.target).value = self.value }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ok(){
        let mut trail = Rc::new(RefCell::new(Trail::new()));
        let mut a = ReversibleInt::new(Rc::clone(&trail), 0);

        assert_eq!(trail.borrow().level(), 0);
        assert_eq!(a.get_value(), 0);

        trail.borrow_mut().push();
        assert_eq!(trail.borrow().level(), 1);
        assert_eq!(a.get_value(), 0);

        a.set_value(1);
        assert_eq!(a.get_value(), 1);

        trail.borrow_mut().push();
        assert_eq!(trail.borrow().level(), 2);
        assert_eq!(a.get_value(), 1);

        a.set_value(2);
        assert_eq!(a.get_value(), 2);

        a.set_value(42);
        assert_eq!(a.get_value(), 42);

        trail.borrow_mut().pop();
        assert_eq!(a.get_value(), 1);
        assert_eq!(trail.borrow().level(), 1);

        trail.borrow_mut().pop();
        assert_eq!(a.get_value(), 0);
        assert_eq!(trail.borrow().level(), 0);
    }
}