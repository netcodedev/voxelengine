use crate::core::{
    renderer::{
        plane::{PlaneBuilder, PlaneRenderer},
        text::{Fonts, Text},
        ui::{container::ContainerBuilder, UIElement},
    },
    scene::Scene,
};

use super::{Panel, PanelBuilder};

impl UIElement for Panel {
    fn render(&mut self, scene: &mut Scene) {
        PlaneRenderer::render(&self.plane);
        PlaneRenderer::render(&self.header_plane);
        self.text.set_content(self.title.clone());
        self.text.render_at(
            (self.offset.0 + self.position.0 + 8.0) as i32,
            (self.offset.1 + self.position.1 + 2.0) as i32,
        );
        self.content.render(scene);
    }

    fn handle_events(
        &mut self,
        scene: &mut Scene,
        window: &mut glfw::Window,
        glfw: &mut glfw::Glfw,
        event: &glfw::WindowEvent,
    ) -> bool {
        // test if click is within bounds
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, glfw::Action::Press, _) => {
                let (x, y) = window.get_cursor_pos();
                if x as f32 >= self.offset.0 + self.position.0
                    && x as f32 <= self.offset.0 + self.position.0 + self.size.0
                    && y as f32 >= self.offset.1 + self.position.1
                    && y as f32 <= self.offset.1 + self.position.1 + 20.0
                {
                    // Start dragging
                    self.drag_start = Some((x, y));
                    self.dragging = true;
                    return true;
                }
            }
            glfw::WindowEvent::MouseButton(
                glfw::MouseButton::Button1,
                glfw::Action::Release,
                _,
            ) => {
                // Stop dragging
                self.dragging = false;
                self.drag_start = None;
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                if *x as f32 >= self.offset.0 + self.position.0
                    && *x as f32 <= self.offset.0 + self.position.0 + self.size.0
                    && *y as f32 >= self.offset.1 + self.position.1
                    && *y as f32 <= self.offset.1 + self.position.1 + 20.0
                {
                    if !self.is_hovering {
                        window.set_cursor(Some(glfw::Cursor::standard(glfw::StandardCursor::Hand)));
                        self.is_hovering = true;
                        self.header_plane.set_color((0.3, 0.4, 0.6, 1.0));
                    }
                } else if self.is_hovering {
                    window.set_cursor(None);
                    self.is_hovering = false;
                    self.header_plane.set_color((0.2, 0.3, 0.5, 1.0));
                }
                if self.dragging {
                    // Update panel position while dragging
                    if let Some((start_x, start_y)) = self.drag_start {
                        self.position.0 += (*x - start_x) as f32 - self.offset.0;
                        self.position.1 += (*y - start_y) as f32 - self.offset.1;
                        self.drag_start = Some((*x, *y));
                        self.set_offset(self.offset); // update children
                    }
                    return true;
                }
            }
            _ => (),
        }
        self.content.handle_events(scene, window, glfw, event)
    }

    fn add_children(&mut self, children: Vec<Box<dyn UIElement>>) {
        self.content.add_children(children);
    }

    fn set_offset(&mut self, offset: (f32, f32)) {
        self.offset = offset;
        self.plane.set_position((
            self.position.0 + self.offset.0,
            self.position.1 + self.offset.1,
            0.0,
        ));
        self.header_plane.set_position((
            self.position.0 + self.offset.0,
            self.position.1 + self.offset.1,
            0.0,
        ));
        self.content.set_offset((
            self.offset.0 + self.position.0,
            self.offset.1 + self.position.1 + 20.0,
        ));
    }

    fn get_size(&self) -> (f32, f32) {
        self.size
    }
}

impl Panel {
    pub fn new(position: (f32, f32, f32), size: (f32, f32), title: String) -> Self {
        let mut content = ContainerBuilder::new()
            .position(0.0, 0.0)
            .size(size.0, size.1 - 40.0)
            .build();
        content.set_offset((position.0, position.1 + 20.0));
        let plane = PlaneBuilder::new()
            .position((position.0, position.1, 0.0))
            .size((size.0, size.1))
            .color((0.2, 0.2, 0.2, 1.0))
            .border_radius_uniform(5.0)
            .border_thickness(1.0)
            .build();
        let header_plane = PlaneBuilder::new()
            .position((position.0, position.1, 0.0))
            .size((size.0, 20.0))
            .color((0.2, 0.3, 0.5, 1.0))
            .border_radius((5.0, 5.0, 0.0, 0.0))
            .border_thickness(1.0)
            .build();
        Self {
            position,
            size,
            title: title.clone(),
            content,
            text: Text::new(Fonts::RobotoMono, 0, 0, 16.0, title),
            offset: (0.0, 0.0),
            drag_start: None,
            dragging: false,
            is_hovering: false,
            plane,
            header_plane,
        }
    }
}

impl PanelBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            size: (0.0, 0.0),
            title: title.to_string(),
            children: Vec::new(),
        }
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y, 0.0);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    pub fn add_child(mut self, child: Box<dyn UIElement>) -> Self {
        self.children.push(child);
        self
    }

    pub fn build(self) -> Panel {
        let mut panel = Panel::new(self.position, self.size, self.title.clone());
        panel.add_children(self.children);
        panel
    }
}