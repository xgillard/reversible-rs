//! This submodule provides you with the types you will be willing to use
//! as basic types to implement the variables of your CP model.
//! Namely, this submodule provides the following types:
//!   - Reversible (an object (primitive) whose value can be automagically reset.

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use ::context::Trail;

/// This is the reversible object abstraction. It holds a reference to its
/// parent context. This way, it will be able to post entries on the trail.
pub struct Reversible<T> {
    trail: Rc<RefCell<Trail>>,
    clock: usize,
    value: T
}

impl<T> Reversible<T>
    where T: Copy + PartialEq + 'static {
    /// Creates a new reversible object associated with the given trail and
    /// initialized with the given value.
    pub fn new(trail: Rc<RefCell<Trail>>, initial: T) -> Reversible<T> {
        let clock = trail.borrow().clock();
        Reversible {
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
            self.trail.borrow_mut().push_on_trail(Box::new(move || unsafe {(*me).value = val }));
        }
    }

    /// Returns the current value of the reversible object
    pub fn get_value(&self) -> T {
        self.value
    }

    /// Changes the current value of the reversible object
    pub fn set_value(&mut self, v: T) {
        if v != self.value {
            self.trail();
            self.value = v;
        }
        self.value = v
    }

}

impl<T: fmt::Display> fmt::Display for Reversible<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reversible({})", self.value)
    }
}

#[cfg(test)]
mod test {
    extern crate rand;
    use super::*;

    #[test]
    fn test_ok(){
        let trail = Rc::new(RefCell::new(Trail::new()));
        let mut a = Reversible::new(Rc::clone(&trail), 0);

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

    #[test]
    fn test_dynamic() {
        let seed : isize = rand::random();

        let trail = Rc::new(RefCell::new(Trail::new()));
        let mut a = Reversible::new(Rc::clone(&trail), seed);


        trail.borrow_mut().push();
        a.set_value(42);
        trail.borrow_mut().pop();

        assert_eq!(seed, a.get_value());
    }


    #[test]
    fn test_boolean() {
        let seed : isize = rand::random();

        let trail = Rc::new(RefCell::new(Trail::new()));
        let mut a = Reversible::new(Rc::clone(&trail), seed%2 == 0);


        trail.borrow_mut().push();
        a.set_value(false);
        trail.borrow_mut().pop();

        assert_eq!(seed %2 == 0, a.get_value());
    }

    #[test]
    fn test_str() {
        let trail = Rc::new(RefCell::new(Trail::new()));
        let mut a = Reversible::new(Rc::clone(&trail), "Coucou");
        assert_eq!("Coucou", a.get_value());

        trail.borrow_mut().push();
        a.set_value("je vais dormir");
        trail.borrow_mut().push();
        a.set_value("maintenant");

        assert_eq!("maintenant", a.get_value());
        trail.borrow_mut().pop_all();
        assert_eq!("Coucou", a.get_value());
    }
}