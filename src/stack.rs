use crate::types::U256;

/// EVM stack: a LIFO structure holding up to 1024 256-bit values.
///
/// In Yellow Paper notation, this is μ_s (machine state, stack component).
/// The stack grows upward — push appends, pop removes from the top.
// ANCHOR: stack_struct
pub struct Stack {
    data: Vec<U256>,
}
// ANCHOR_END: stack_struct

// ANCHOR: stack_error
/// Errors that can occur during stack operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackError {
    /// Attempted to push onto a full stack (1024 elements).
    Overflow,
    /// Attempted to pop from or peek into an empty stack.
    Underflow,
}
// ANCHOR_END: stack_error

impl std::fmt::Display for StackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StackError::Overflow => write!(f, "stack overflow: maximum depth 1024 exceeded"),
            StackError::Underflow => write!(f, "stack underflow: not enough elements"),
        }
    }
}

impl std::error::Error for StackError {}

/// Maximum stack depth as defined by the EVM specification.
pub const MAX_STACK_DEPTH: usize = 1024;

impl Stack {
    /// Create a new, empty stack.
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(64), // reasonable initial capacity
        }
    }

    /// Push a value onto the top of the stack.
    ///
    /// Returns `StackError::Overflow` if the stack already has 1024 elements.
    // ANCHOR: stack_push_pop
    pub fn push(&mut self, value: U256) -> Result<(), StackError> {
        if self.data.len() >= MAX_STACK_DEPTH {
            return Err(StackError::Overflow);
        }
        self.data.push(value);
        Ok(())
    }

    /// Pop the top value from the stack.
    ///
    /// Returns `StackError::Underflow` if the stack is empty.
    pub fn pop(&mut self) -> Result<U256, StackError> {
        self.data.pop().ok_or(StackError::Underflow)
    }
    // ANCHOR_END: stack_push_pop

    /// Peek at the top value without removing it.
    pub fn peek(&self) -> Result<&U256, StackError> {
        self.data.last().ok_or(StackError::Underflow)
    }

    /// Peek at the value at `depth` positions from the top (0 = top).
    pub fn peek_at(&self, depth: usize) -> Result<&U256, StackError> {
        if depth >= self.data.len() {
            return Err(StackError::Underflow);
        }
        Ok(&self.data[self.data.len() - 1 - depth])
    }

    /// Swap the top element with the element at `depth` positions below it.
    ///
    /// `depth` must be in range 1..=16 (for SWAP1..SWAP16).
    pub fn swap(&mut self, depth: usize) -> Result<(), StackError> {
        let len = self.data.len();
        if depth == 0 || depth >= len {
            return Err(StackError::Underflow);
        }
        self.data.swap(len - 1, len - 1 - depth);
        Ok(())
    }

    /// Duplicate the element at `depth` positions from the top and push it.
    ///
    /// `depth` must be in range 1..=16 (for DUP1..DUP16).
    pub fn dup(&mut self, depth: usize) -> Result<(), StackError> {
        if depth == 0 || depth > self.data.len() {
            return Err(StackError::Underflow);
        }
        let value = self.data[self.data.len() - depth];
        self.push(value)
    }

    /// Number of elements currently on the stack.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Exercise 1.5: Stack push and pop
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_5_push_and_pop_lifo() {
        let mut stack = Stack::new();
        stack.push(U256::from(1u64)).unwrap();
        stack.push(U256::from(2u64)).unwrap();
        stack.push(U256::from(3u64)).unwrap();

        assert_eq!(stack.pop().unwrap(), U256::from(3u64));
        assert_eq!(stack.pop().unwrap(), U256::from(2u64));
        assert_eq!(stack.pop().unwrap(), U256::from(1u64));
    }

    #[test]
    fn ex_1_5_pop_empty_returns_underflow() {
        let mut stack = Stack::new();
        assert_eq!(stack.pop(), Err(StackError::Underflow));
    }

    #[test]
    fn ex_1_5_peek_does_not_remove() {
        let mut stack = Stack::new();
        stack.push(U256::from(42u64)).unwrap();
        assert_eq!(*stack.peek().unwrap(), U256::from(42u64));
        assert_eq!(stack.len(), 1);
    }

    // ---------------------------------------------------------------
    // Exercise 1.6: Stack depth limit
    // ---------------------------------------------------------------
    #[test]
    fn ex_1_6_max_depth_is_1024() {
        let mut stack = Stack::new();
        for i in 0..1024u64 {
            stack.push(U256::from(i)).unwrap();
        }
        assert_eq!(stack.len(), 1024);

        // The 1025th push must fail.
        assert_eq!(stack.push(U256::from(1025u64)), Err(StackError::Overflow));
    }

    // ---------------------------------------------------------------
    // Additional: swap and dup
    // ---------------------------------------------------------------
    #[test]
    fn swap_top_with_second() {
        let mut stack = Stack::new();
        stack.push(U256::from(1u64)).unwrap();
        stack.push(U256::from(2u64)).unwrap();
        stack.swap(1).unwrap();

        assert_eq!(stack.pop().unwrap(), U256::from(1u64));
        assert_eq!(stack.pop().unwrap(), U256::from(2u64));
    }

    #[test]
    fn dup_top_element() {
        let mut stack = Stack::new();
        stack.push(U256::from(7u64)).unwrap();
        stack.dup(1).unwrap();

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop().unwrap(), U256::from(7u64));
        assert_eq!(stack.pop().unwrap(), U256::from(7u64));
    }
}
