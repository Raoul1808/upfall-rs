use egui_tetra::egui;
use tetra::{
    graphics::{
        self,
        scaling::{ScalingMode, ScreenScaler},
        Camera, Color,
    },
    input::{self, Key},
    math::Vec2,
    Event,
};

use crate::{level::Level, world::World, Assets, Scene};

use super::Transition;

fn buf_to_col(buf: [f32; 3]) -> Color {
    Color {
        r: buf[0],
        g: buf[1],
        b: buf[2],
        a: 1.,
    }
}

pub struct GameScene {
    world: World,
    color_a: Color,
    color_b: Color,
    camera: Camera,
    scaler: ScreenScaler,
    dt: f32,
    color_a_buf: [f32; 3],
    color_b_buf: [f32; 3],
}

impl GameScene {
    const INNER_SIZE: Vec2<i32> = Vec2::new(640, 360);
    pub fn new(ctx: &mut tetra::Context, level: Level) -> tetra::Result<GameScene> {
        Ok(GameScene {
            world: World::new(level),
            color_a: Color::BLACK,
            color_b: Color::WHITE,
            camera: Camera::new(Self::INNER_SIZE.x as f32, Self::INNER_SIZE.y as f32),
            scaler: ScreenScaler::with_window_size(
                ctx,
                Self::INNER_SIZE.x,
                Self::INNER_SIZE.y,
                ScalingMode::ShowAll,
            )?,
            dt: 0.,
            color_a_buf: [0.; 3],
            color_b_buf: [1.; 3],
        })
    }
}

impl Scene for GameScene {
    fn event(&mut self, _ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Escape) {
            return Ok(Transition::Pop);
        }

        self.dt += tetra::time::get_delta_time(ctx).as_secs_f32();
        self.world.update(ctx);
        self.camera.position = self.world.player_pos();
        let world_rect = self.world.get_world_rect();
        let cam_rect = self.camera.visible_rect();
        if cam_rect.left() < world_rect.left() {
            self.camera.position.x += world_rect.left() - cam_rect.left();
        }
        if cam_rect.right() > world_rect.right() {
            self.camera.position.x += world_rect.right() - cam_rect.right();
        }
        if cam_rect.top() < world_rect.top() {
            self.camera.position.y += world_rect.top() - cam_rect.top();
        }
        if cam_rect.bottom() > world_rect.bottom() {
            self.camera.position.y += world_rect.bottom() - cam_rect.bottom();
        }
        self.camera.update();
        Ok(Transition::None)
    }

    fn egui_layout(
        &mut self,
        _ctx: &mut tetra::Context,
        egui_ctx: &egui_tetra::egui::CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        egui::Window::new("Background Color").show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Color A");
                if ui.color_edit_button_rgb(&mut self.color_a_buf).changed() {
                    self.color_a = buf_to_col(self.color_a_buf);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Color B");
                if ui.color_edit_button_rgb(&mut self.color_b_buf).changed() {
                    self.color_b = buf_to_col(self.color_b_buf);
                }
            });
        });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        assets
            .shader
            .set_uniform(ctx, "u_color_a", self.color_a.with_alpha(1.));
        assets
            .shader
            .set_uniform(ctx, "u_color_b", self.color_b.with_alpha(1.));
        assets.shader.set_uniform(
            ctx,
            "u_circle_offset",
            self.camera.visible_rect().top_left(),
        );
        assets
            .shader
            .set_uniform(ctx, "u_resolution", Self::INNER_SIZE.as_());
        assets
            .shader
            .set_uniform(ctx, "u_scale_factor", self.scaler.scale_factor());
        let scale_offset = self.scaler.unproject(Vec2::zero());
        assets
            .shader
            .set_uniform(ctx, "u_scale_offset", scale_offset);
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::set_transform_matrix(ctx, self.camera.as_matrix());
        graphics::clear(ctx, Color::BLACK);
        self.world.draw(ctx, assets);
        graphics::reset_transform_matrix(ctx);
        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        graphics::set_shader(ctx, &assets.shader);
        self.scaler.draw(ctx);
        graphics::reset_shader(ctx);
        Ok(())
    }
}
