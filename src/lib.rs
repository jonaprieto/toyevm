/// Marks an exercise body that the student must implement.
/// Tests using this macro will panic with a hint pointing to the relevant chapter.
#[macro_export]
macro_rules! todo_exercise {
    ($msg:literal) => {
        panic!("\n\n  Exercise not yet implemented.\n  Hint: {}\n\n", $msg)
    };
    () => {
        panic!("\n\n  Exercise not yet implemented. Read the book chapter for this module.\n\n")
    };
}

pub mod gas;
pub mod interpreter;
pub mod memory;
pub mod opcode;
pub mod stack;
pub mod types;
