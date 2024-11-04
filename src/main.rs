use cgmath::Deg;
use glfw::{Glfw, WindowEvent};

mod core;
mod terrain;
use core::{
    application::{Application, Layer},
    camera::{Camera, CameraController, Projection},
    entity::{
        component::{
            camera_component::CameraComponent, debug_component::DebugController,
            model_component::ModelComponent,
        },
        Entity,
    },
    model::ModelBuilder,
    renderer::ui::{UIRenderer, UI},
    scene::Scene,
};
use terrain::{dual_contouring::DualContouringChunk, Terrain};

fn main() {
    let mut application = Application::new(1280, 720, "Engine");
    if let Ok(layer) = WorldLayer::new(1280, 720) {
        application.add_layer(Box::new(layer));
        application.start();
    }
}

struct WorldLayer {
    scene: Scene,
    ui: UIRenderer,
}

impl WorldLayer {
    pub fn new(width: u32, height: u32) -> Result<WorldLayer, Box<dyn std::error::Error>> {
        let mut scene = Scene::new();
        let camera = Camera::new((-119.4, 52.7, -30.0), Deg(-138.0), Deg(-17.0));
        let projection: Projection = Projection::new(width, height, Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(10.0, 1.0);
        let mut entity = Entity::new();
        entity.add_component(CameraComponent::new(camera, projection, camera_controller));
        scene.add_entity(entity);
        let ui = UIRenderer::new();

        let mut debug = Entity::new();
        debug.add_component(DebugController::new());
        scene.add_entity(debug);

        let mut terrain_entity = Entity::new();
        terrain_entity.add_component(Terrain::<DualContouringChunk>::new());
        scene.add_entity(terrain_entity);

        let mut model_entity = Entity::new();
        let mut model = ModelBuilder::new("Mannequin.fbx")?
            .with_animation("idle", "Idle.fbx")
            .with_animation("walk", "Walk.fbx")
            .with_animation("run", "Run.fbx")
            .build();
        model.init();
        model.blend_animations("walk", "run", 0.5, true);
        model.play_animation("idle");
        model_entity.add_component(ModelComponent::new(model));
        scene.add_entity(model_entity);

        Ok(Self { scene, ui })
    }
}

impl Layer for WorldLayer {
    fn on_attach(&mut self) {
        self.ui.add(UI::panel("Camera controls", |builder| {
            builder
                .position(10.0, 130.0)
                .add_child(UI::text("Camera Speed", 16.0, |b| b))
                .add_child(UI::input(|input| {
                    input
                        .size(190.0, 26.0)
                        .get_fn(|scene| {
                            let camera_controller = scene
                                .get_component::<CameraComponent>()
                                .unwrap()
                                .get_camera_controller();
                            camera_controller.get_speed().to_string()
                        })
                        .set_fn(move |scene, v| {
                            let camera_controller = scene
                                .get_component_mut::<CameraComponent>()
                                .unwrap()
                                .get_camera_controller_mut();
                            match v.parse::<f32>() {
                                Ok(v) => camera_controller.set_speed(v),
                                Err(_) => {}
                            }
                        })
                }))
                .add_child(UI::button(
                    "Reset Speed",
                    Box::new(move |scene| {
                        let camera_controller = scene
                            .get_component_mut::<CameraComponent>()
                            .unwrap()
                            .get_camera_controller_mut();
                        camera_controller.set_speed(10.0);
                    }),
                    |b| b,
                ))
        }));
    }

    fn on_update(&mut self, delta_time: f64) {
        self.scene.update(delta_time);
        self.scene.render();

        self.ui.render(&mut self.scene);
    }

    fn on_event(&mut self, glfw: &mut Glfw, window: &mut glfw::Window, event: &WindowEvent) {
        if self.ui.handle_events(&mut self.scene, window, glfw, &event) {
            return;
        }
        self.scene.handle_event(glfw, window, event);
    }

    fn get_name(&self) -> &str {
        "World"
    }
}
