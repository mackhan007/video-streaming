//! Shared domain primitives for EMS / IMS services.
//! Keep this crate free of HTTP, SQL, and cloud SDK types (logging bootstrap excepted).

pub mod logging;

mod events;
mod video_id;
mod video_status;

pub use events::VideoUploaded;
pub use video_id::VideoId;
pub use video_status::VideoStatus;