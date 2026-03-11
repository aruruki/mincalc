use serde::{Deserialize, Serialize};

use crate::state::Concentrate;

const STORAGE_KEY: &str = "mincalc_concentrates";

#[derive(Serialize, Deserialize)]
struct PersistedState {
    concentrates: Vec<Concentrate>,
    next_id: u32,
}

/// Persist the concentrate list and next_id counter to localStorage.
/// Silently does nothing if localStorage is unavailable.
pub fn save_concentrates(concentrates: &[Concentrate], next_id: u32) {
    let Ok(json) = serde_json::to_string(&PersistedState {
        concentrates: concentrates.to_vec(),
        next_id,
    }) else {
        return;
    };

    let Some(storage) = local_storage() else {
        return;
    };

    let _ = storage.set_item(STORAGE_KEY, &json);
}

/// Load the concentrate list and next_id counter from localStorage.
/// Returns `None` if nothing is stored or the data cannot be deserialized.
pub fn load_concentrates() -> Option<(Vec<Concentrate>, u32)> {
    let storage = local_storage()?;
    let json = storage.get_item(STORAGE_KEY).ok()??;
    let state: PersistedState = serde_json::from_str(&json).ok()?;
    Some((state.concentrates, state.next_id))
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}
