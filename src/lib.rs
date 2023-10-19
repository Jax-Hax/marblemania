pub mod camera;
pub mod structs;
pub mod model;
pub mod loader;
pub mod shader;
pub mod state;
pub mod texture;
pub mod window;
pub mod prefabs;
pub mod primitives;
pub mod collision;
pub mod event;
mod render;
pub mod prelude {
    pub use crate::{
        structs::{Instance,InstanceRaw},
        event::{run_event_loop,delta_time_to_seconds},
        state::State,
        structs::Vertex,
        camera::Camera
    };
    #[cfg(target_arch = "wasm32")]
    pub use wasm_bindgen::prelude::*;
    pub use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
}