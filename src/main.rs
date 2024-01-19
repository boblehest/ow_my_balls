use rapier2d::prelude::*;
use miniquad::EventHandler;
use macroquad::prelude::*;
use macroquad::input::utils::*;

// in world units (not pixels)
const LINE_WIDTH : f32 = 2.0; // used for ground and walls
const LEVEL_WIDTH : f32 = 40.0;
const LEVEL_HEIGHT : f32 = 30.0;

struct Model {
    island_manager: IslandManager,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    step_physics_fn: Box<dyn FnMut(&mut IslandManager, &mut RigidBodySet, &mut ColliderSet) -> ()>,
    camera: Camera2D,
}

impl EventHandler for Model {
    fn update(&mut self) {
        (self.step_physics_fn)(&mut self.island_manager, &mut self.rigid_body_set, &mut self.collider_set);
    }

    fn draw(&mut self) {
        clear_background(BLACK);

        // floor
        draw_rectangle(0.0, 0.0, LEVEL_WIDTH, LINE_WIDTH, ORANGE);
        // left wall
        draw_rectangle(0.0, 0.0, LINE_WIDTH, LEVEL_HEIGHT, ORANGE);
        // right wall
        draw_rectangle(LEVEL_WIDTH - LINE_WIDTH, 0.0, LINE_WIDTH, LEVEL_HEIGHT, ORANGE);

        for handle in self.island_manager.active_dynamic_bodies() {
            let ball_body = &self.rigid_body_set[*handle];
            let color : Color = match ball_body.user_data {
                42 => BLUE,
                _ => ORANGE,
            };
            draw_circle(
                ball_body.translation().x,
                ball_body.translation().y,
                0.2,
                color);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        button: MouseButton,
        x: f32,
        y: f32
    ) {
        match button {
            miniquad::MouseButton::Left => {
                let click_position = self.camera.screen_to_world(Vec2 { x, y });
                let rigid_body = RigidBodyBuilder::dynamic()
                    .translation(vector![click_position.x, click_position.y])
                    .linvel(vector![0.0, -80.0])
                    .user_data(42)
                    .build();
                let collider = ColliderBuilder::ball(0.2).restitution(0.9).mass(50.0).build();
                let ball_body_handle = self.rigid_body_set.insert(rigid_body);
                self.collider_set.insert_with_parent(collider, ball_body_handle, &mut self.rigid_body_set);
            },
            _ => {},
        }
    }

}

fn init_model(camera: Camera2D) -> Model {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create the ground. */
    let collider = ColliderBuilder::cuboid(LEVEL_WIDTH * 0.5, LINE_WIDTH * 0.5)
        .translation(vector![LEVEL_WIDTH * 0.5, LINE_WIDTH * 0.5])
        .build();
    /* walls */
    let collider_wall_l = ColliderBuilder::cuboid(LINE_WIDTH * 0.5, LEVEL_HEIGHT * 0.5)
        .translation(vector![LINE_WIDTH * 0.5, LEVEL_HEIGHT * 0.5])
        .build();
    let collider_wall_r = ColliderBuilder::cuboid(LINE_WIDTH * 0.5, LEVEL_HEIGHT * 0.5)
        .translation(vector![LEVEL_WIDTH - LINE_WIDTH * 0.5, LEVEL_HEIGHT * 0.5])
        .build();

    collider_set.insert(collider);
    collider_set.insert(collider_wall_l);
    collider_set.insert(collider_wall_r);

    /* Create the bouncing ball. */
    for i in 0..9001 {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![LEVEL_WIDTH / 2.0 + 0.0001*i as f32, 10.0+i as f32])
            .build();
        let collider = ColliderBuilder::ball(0.2).restitution(0.9).build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);
    }

    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, -9.81];
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let island_manager = IslandManager::new();
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    let step_physics_fn = Box::new(move |island_manager: &mut IslandManager, rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet| {
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            rigid_body_set,
            collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );
    });

    Model {
        island_manager,
        rigid_body_set,
        collider_set,
        step_physics_fn,
        camera,
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let camera = Camera2D::from_display_rect(Rect::new(0., 0., LEVEL_WIDTH, LEVEL_HEIGHT));
    set_camera(&camera);
    let mut model = init_model(camera);
    let input_subscription = register_input_subscriber();
    let mut last_update = get_time();
    loop {
        let frame_start = get_time();
        let time_since_update = frame_start - last_update;
        if time_since_update > 1.0/50.0 {
            repeat_all_miniquad_input(&mut model, input_subscription);
            model.update();
            last_update = frame_start;
        }
        model.draw();
        next_frame().await
    }
}
