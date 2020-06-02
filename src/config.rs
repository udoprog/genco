/// A trait that must be implemented for all custom configurations.
pub trait Config {
    /// Determines the indentation level.
    fn indentation(&mut self) -> usize {
        2
    }
}

impl Config for () {}
