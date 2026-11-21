pub mod bridge;
pub mod cli;
pub mod codegen;
pub mod context;
pub mod manifest;
pub mod ops;
pub mod ui;

use std::error::Error;
use std::result::Result as StdResult;

pub use cli::run;
pub use context::{Context, Parts};
pub use manifest::{Entry, EntryKind, Meta, Plugin, Setting, SettingKind};
pub use ops::Op;

pub type Result<T> = StdResult<T, Box<dyn Error>>;
