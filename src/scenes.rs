use tetra::{
    graphics::{Color, DrawParams},
    input::{self, Key},
    math::{Rect, Vec2},
};

use crate::Assets;

pub trait Scene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result;
    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result;
}

pub struct Player {
    pub position: Vec2<f32>,
    pub velocity: Vec2<f32>,
}

const PLAYER_SQUARE: f32 = 32.;

impl Player {
    pub fn new() -> Player {
        Player {
            position: Vec2::new(100., 100.),
            velocity: Vec2::default(),
        }
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        const MAX_FALL_SPEED: f32 = 10.;
        const GRAVITY: f32 = 0.5;
        const JUMP_FORCE: f32 = 6.;

        let mut x_vel = 0.;
        if input::is_key_down(ctx, Key::Left) {
            x_vel = -2.;
        }
        if input::is_key_down(ctx, Key::Right) {
            x_vel = 2.;
        }
        self.velocity.x = x_vel;

        self.velocity.y += GRAVITY;
        if self.velocity.y > MAX_FALL_SPEED {
            self.velocity.y = MAX_FALL_SPEED;
        }

        if input::is_key_pressed(ctx, Key::Space) {
            self.velocity.y = -JUMP_FORCE;
        }
    }

    pub fn solve_collision_y(&mut self, rect: &Rect<f32, f32>) {
        self.position.y += self.velocity.y;
        let hbox = self.make_hbox();
        if rect.collides_with_rect(hbox) {
            if self.velocity.y < 0. {
                self.position.y = rect.y + rect.h;
                self.velocity.y = 0.;
            }
            if self.velocity.y > 0. {
                self.position.y = rect.y - PLAYER_SQUARE;
                self.velocity.y = 0.;
            }
        }
    }

    pub fn solve_collision_x(&mut self, rect: &Rect<f32, f32>) {
        self.position.x += self.velocity.x;
        let hbox = self.make_hbox();
        if rect.collides_with_rect(hbox) {
            if self.velocity.x > 0. {
                self.position.x = rect.x - PLAYER_SQUARE;
                self.velocity.x = 0.;
            }
            if self.velocity.x < 0. {
                self.position.x = rect.x + rect.w;
                self.velocity.x = 0.;
            }
        }
    }

    pub fn make_hbox(&self) -> Rect<f32, f32> {
        Rect::new(
            self.position.x,
            self.position.y,
            PLAYER_SQUARE,
            PLAYER_SQUARE,
        )
    }
}

pub struct GameScene {
    player: Player,
    rects: Vec<Rect<f32, f32>>,
}

impl GameScene {
    pub fn new() -> GameScene {
        GameScene {
            player: Player::new(),
            rects: vec![
                Rect::new(50., 500., 700., 100.),
                Rect::new(350., 300., 150., 50.),
                Rect::new(0., 0., 1000., 50.),
            ],
        }
    }
}

impl Scene for GameScene {
    fn update(&mut self, ctx: &mut tetra::Context) -> tetra::Result {
        let m_pos = input::get_mouse_position(ctx);
        if input::is_mouse_button_pressed(ctx, input::MouseButton::Left) {
            self.player.position.x = m_pos.x;
            self.player.position.y = m_pos.y;
        }
        self.player.update(ctx);

        for rect in &self.rects {
            self.player.solve_collision_y(rect);
        }

        for rect in &self.rects {
            self.player.solve_collision_x(rect);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(self.player.position)
                .scale(Vec2::new(32., 32.))
                .color(Color::RED),
        );
        for rect in &self.rects {
            assets.pixel.draw(
                ctx,
                DrawParams::new()
                    .position(Vec2::new(rect.x, rect.y))
                    .scale(Vec2::new(rect.w, rect.h))
                    .color(Color::BLACK),
            );
        }
        Ok(())
    }
}
