use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyBindType<T> {
    ToggleOnce(T),
    ToggleTwice(T),
    Value(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBind<T> {
    key_bind_type: KeyBindType<T>,
    address: String,
}

// impl<T> Default for KeyBind<T> {
//     fn default() -> Self {
//         Self {
//             key_bind_type: KeyBindType::ToggleOnce(),
//             address: "/".to_string(),
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keybinds: hashbrown::HashMap<raw_input::Key, KeyBind<String>>,
}

// impl Default for Config {
//     fn default() -> Self {
//         let mut keybinds: hashbrown::HashMap<raw_input::Key, KeyBind<T>> =
//             hashbrown::HashMap::new();
//
//         keybinds.insert(
//             raw_input::Key::ControlRight,
//             KeyBind {
//                 key_bind_type: KeyBindType::Once,
//                 address: "/avatar/parameters/Hypnosis".to_string(),
//             },
//         );
//
//         Self { keybinds }
//     }
// }
