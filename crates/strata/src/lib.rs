// Strata Engine - A voxel game engine for Daggerfall-like experiences

pub mod error;
pub mod renderer;
pub mod window;

pub use error::{Result, StrataError};

pub struct Engine {
    // TODO
}

impl Engine {
    pub fn new() -> Result<Self> {
        todo!("Create engine")
    }

    pub fn run(self) -> Result<()> {
        todo!("Run engine loop")
    }
}
