//! The only target difference above the HTTP client: native hosts run tasks on a
//! multi-threaded runtime, so tasks and sinks must be `Send`; the browser has one thread.

use std::{future::Future, pin::Pin};

#[cfg(not(target_arch = "wasm32"))]
pub type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
#[cfg(target_arch = "wasm32")]
pub type Task = Pin<Box<dyn Future<Output = ()>>>;

#[cfg(not(target_arch = "wasm32"))]
pub type Sink = Box<dyn Fn(Vec<u8>) + Send>;
#[cfg(target_arch = "wasm32")]
pub type Sink = Box<dyn Fn(Vec<u8>)>;

/// Runs a task to completion on the host's executor.
pub type Spawn = fn(Task);
