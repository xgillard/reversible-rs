//! This module provides the trailing context at the heart of a trailing solver.
//!
//! Its code is *heavily* inspired from that of minicp (and Oscar, and Comet, ...)
use std::boxed::Box;

/// This structure implements the trail, aka the reversible context.
pub struct Trail {
    clock : usize,
    trail : Vec< Box<dyn FnMut()>  >,
    limit : Vec< usize >
}

impl Trail {
    /// Create a new reversible context.
    /// The current level is -1
    pub fn new() -> Trail {
        Trail {
            clock: 0,
            trail: vec![],
            limit: vec![]
        }
    }

    /// Callback to remember what needs to be undone upon restoration of the state
    pub fn push_on_trail(&mut self, entry: Box<dyn FnMut()> ) {
        self.trail.push(entry)
    }

    /// Saves the current state so that it can be restored
    /// with a pop. Increases the level by one.
    pub fn push(&mut self) {
        self.clock += 1;
        self.limit.push( self.trail.len() );
    }

    /// Restores state as it was at level()-1
    /// Decrease the level by 1
    pub fn pop(&mut self) {
        let sz = self.limit.pop().unwrap_or(0);
        while self.trail.len() > sz {
            self.trail.pop().unwrap()();
        }
        self.clock += 1;
    }

    /// Restores the state as it was at level 0 (first push)
    /// The level is now -1.
    ///
    /// Note: You'll probably want to push after this operation.
    pub fn pop_all(&mut self) {
        self.pop_until(0)
    }

    /// Restores the state as it was at level
    pub fn pop_until(&mut self, level: usize) {
        while self.level() > level {
            self.pop()
        }
    }

    /// Returns the current level
    pub fn level(&self) -> usize {
        self.limit.len()
    }

    /// Returns the current value of the clock
    pub fn clock(&self) -> usize {
        self.clock
    }
}
