use crate::core::{renderer::text::Text, scene::Scene};

pub mod input;

type GetFn = dyn Fn(&mut Scene) -> String;
type SetFn = dyn FnMut(&mut Scene, String);

pub struct Input {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub offset: (f32, f32),
    pub is_hovering: bool,
    pub is_focused: bool,
    pub content: String,
    text: Text,
    get_fn: Option<Box<GetFn>>,
    set_fn: Option<Box<SetFn>>,
}

pub struct InputBuilder {
    position: (f32, f32),
    size: (f32, f32),
    content: String,
    get_fn: Option<Box<GetFn>>,
    set_fn: Option<Box<SetFn>>,
}
