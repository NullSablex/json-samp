use samp::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::logger::Logger;

pub struct Plugin {
    /// Active JSON documents, keyed by the ID handed back to the natives.
    pub pool: HashMap<i32, Value>,
    /// Next ID to be allocated by `alloc_id`.
    pub next_id: i32,
}

impl Plugin {
    pub fn new() -> Self {
        Logger::init();
        Plugin {
            pool: HashMap::new(),
            next_id: 1,
        }
    }
}

impl SampPlugin for Plugin {
    fn on_load(&mut self) {}

    fn on_unload(&mut self) {
        Logger::info("Plugin unloaded.");
    }

    fn on_omp_ready(&mut self) {
        Logger::info("Open Multiplayer native mode: all components ready.");
    }

    fn on_component_free(&mut self) {
        Logger::info("Open Multiplayer: a neighbouring component is being unloaded.");
    }
}
