use bevy::{ecs::resource::Resource, input::keyboard::KeyCode};
use fxhash::FxHashMap;

#[derive(Eq, PartialEq, Hash)]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Sprint,

    Primary,
    Secondary,
    Cancel,

    Pause,
    Build,
}

#[derive(Resource)]
pub struct Controls(pub FxHashMap<Action, KeyCode>);

impl Default for Controls {
    fn default() -> Self {
        let mut map = FxHashMap::default();

        map.insert(Action::Forward, KeyCode::KeyW);
        map.insert(Action::Backward, KeyCode::KeyS);
        map.insert(Action::Left, KeyCode::KeyA);
        map.insert(Action::Right, KeyCode::KeyD);
        map.insert(Action::Jump, KeyCode::Space);
        map.insert(Action::Sprint, KeyCode::ShiftLeft);

        map.insert(Action::Primary, KeyCode::KeyE);
        map.insert(Action::Secondary, KeyCode::KeyF);
        map.insert(Action::Cancel, KeyCode::KeyQ);

        map.insert(Action::Pause, KeyCode::Escape);
        map.insert(Action::Build, KeyCode::KeyB);

        Self(map)
    }
}

impl Controls {
    pub fn get(&self, action: Action) -> KeyCode {
        *self.0.get(&action).unwrap_or(&KeyCode::Digit0)
    }

    pub fn print(&self, action: Action) -> String {
        let key = *self.0.get(&action).unwrap_or(&KeyCode::Digit0);
        let debug = format!("{:?}", key);
        if debug.starts_with("Key") {
            debug.chars().last().unwrap().into()
        } else {
            debug
        }
    }
}
