mod parser;
pub mod types;

pub use parser::{load_profile, ProfileError};
pub use types::{GlideProfile, Waypoint};
