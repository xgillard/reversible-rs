//! This submodule provides you with the types you will be willing to use
//! as basic types to implement the variables of your CP model.
//! Namely, this submodule provides the following types:
//!   - Reversible (an object (primitive) whose value can be automagically reset.

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::fmt;

use ::context::Trail;

/// This is the reversible object abstraction. It holds a reference to its
/// parent context. This way, it will be able to post entries on the trail.
///
/// # Implementation Notes
/// The use of a lifetime <'a> as well as smart pointers Rc<RefCell> and Rc<Cell>
/// for the trail and value field might seem somewhat cumbersome. However, these
/// are actually simpler than meets the eye.
///
///  - The lifetime _<'a>_ is used to tell the compiler that it needs to ensure
///     that whenever we push some restoration closure on the trail, any references
///     it holds must live at least as long as <'a> (the scope of the trail).
///     Given that the parameter type <T> forces the bound `Copy`, this should
///     in principle never be an issue. (And if you managed to create one such case,
///     the compiler will warn you.)
///
///   - The type _Rc<RefCell<Trail<'a>>>_ of the field `trail` simply means that
///     the value pointed to by `trail` is shared among multiple objects. All of
///     which might possibly need to mutate the trail state at some point of time.
///     Hence, the type `Rc` means that it is a shared reference (reference counted,
///     not atomic ==> DO NOT USE THIS TYPE IN A PARALLEL SOLVER). `RefCell` tells
///     that `trail` can possibly muted. The borrowing rules are enforced at runtime
///     rather than compile time. This means that ANY RACE CONDITION on the trail
///     will provoke a PANIC !
///
///   - Similarly to the `trail` field, the `value` field bears the type _Rc< Cell<T> >_
///     this indicates that value will not be dropped as long as there exists a way
///     to access it. And that it may be mutated by more than one owner. (Again,
///     borrow checking rules are enforced at runtime rather than compile time. And
///     race conditions will trigger a panic!). Indeed, the value field may be mutated
///     either by using the `set_value(x)` method of the Reversible; or by a restoration
///     closure that has been pushed onto the trail.
///
/// All in all, these seemingly odd constructs provide you with an (imho) elegant solution
/// that lets you tackle the difficult problem of transparent state restoration without
/// sacrificing the guarantees provided by Rust. (No need to resort to the use of _unsafe_
/// code).
pub struct Reversible<'a, T>
    where T: Copy + PartialEq + 'a {
    trail: Rc<RefCell<Trail<'a>>>,
    clock: usize,
    value: Rc<Cell<T>>
}

impl<'a, T> Reversible<'a, T>
    where T: Copy + PartialEq + 'a {
    /// Creates a new reversible object associated with the given trail and
    /// initialized with the given value.
    pub fn new(trail: Rc<RefCell<Trail>>, initial: T) -> Reversible<T> {
        let clock = trail.borrow().clock();
        let value = Rc::new(Cell::new(initial));
        Reversible {
            trail,
            clock,
            value
        }
    }

    /// This private method takes care of posting an entry on the trail
    /// so as to easily restore the current state.
    fn trail(&mut self) {
        let trail_time = self.trail.borrow().clock();

        if trail_time != self.clock {
            self.clock = trail_time;

            let val = self.value.get();
            let dst = Rc::clone(&self.value);
            self.trail.borrow_mut().push_on_trail(Box::new(move || dst.set(val)));
        }
    }

    /// Returns the current value of the reversible object
    pub fn get_value(&self) -> T {
        self.value.get()
    }

    /// Changes the current value of the reversible object.
    /// returns the current value
    pub fn set_value(&mut self, v: T) -> T {
        if v != self.value.get() {
            self.trail();
            self.value.set(v);
        }
        self.value.get()
    }

}

impl<'a, T> fmt::Display for Reversible<'a, T>
    where T: fmt::Display + Copy + PartialEq + 'a {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reversible({})", self.value.get())
    }
}


// TODO: I might want to move unit tests somewhere else (in the tests folder)
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