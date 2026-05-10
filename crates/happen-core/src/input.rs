use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    W, A, S, D,
    Q, E, R, F, G,
    Space, LShift, LControl, LAlt,
    Escape,
    Up, Down, Left, Right,
    Num1, Num2, Num3, Num4, Num5,
    Tab, Enter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub struct Input {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_just_pressed: HashSet<MouseButton>,
    pub mouse_delta: (f32, f32),
    pub cursor_locked: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_just_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            cursor_locked: false,
        }
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn key_just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    pub fn mouse_button_pressed(&self, btn: MouseButton) -> bool {
        self.mouse_pressed.contains(&btn)
    }

    pub fn mouse_button_just_pressed(&self, btn: MouseButton) -> bool {
        self.mouse_just_pressed.contains(&btn)
    }

    pub fn press_key(&mut self, key: KeyCode) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    pub fn release_key(&mut self, key: KeyCode) {
        if self.pressed.remove(&key) {
            self.just_released.insert(key);
        }
    }

    pub fn press_mouse(&mut self, btn: MouseButton) {
        if self.mouse_pressed.insert(btn) {
            self.mouse_just_pressed.insert(btn);
        }
    }

    pub fn release_mouse(&mut self, btn: MouseButton) {
        self.mouse_pressed.remove(&btn);
    }

    pub fn accumulate_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    pub fn end_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_delta = (0.0, 0.0);
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
