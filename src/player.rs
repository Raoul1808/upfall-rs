use tetra::{
    input::{self, Key},
    math::{Rect, Vec2},
};

use crate::tilemap::Axis;

#[derive(Debug)]
pub struct Player {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
    portal_traversed: bool,
}

impl Player {
    pub const PLAYER_SQUARE: f32 = 16.;
    pub const HALF_SIZE: Vec2<f32> = Vec2::new(Self::PLAYER_SQUARE / 2., Self::PLAYER_SQUARE / 2.);

    pub fn new(spawn_pos: Vec2<f32>) -> Player {
        Player {
            position: spawn_pos,
            velocity: Vec2::default(),
            portal_traversed: false,
        }
    }

    pub fn update(&mut self, ctx: &mut tetra::Context, flip: bool) {
        self.portal_traversed = false;
        const MAX_FALL_SPEED: f32 = 15.;
        const GRAVITY: f32 = 0.7;
        const JUMP_FORCE: f32 = 11.;
        const WALK_SPEED: f32 = 4.;

        let y_direction = if flip { -1.0 } else { 1.0 };

        let mut x_vel = 0.;
        if input::is_key_down(ctx, Key::Left) {
            x_vel = -WALK_SPEED;
        }
        if input::is_key_down(ctx, Key::Right) {
            x_vel = WALK_SPEED;
        }
        self.velocity.x = x_vel;

        self.velocity.y += GRAVITY * y_direction;
        if self.velocity.y.abs() > MAX_FALL_SPEED {
            self.velocity.y = MAX_FALL_SPEED * y_direction;
        }

        if input::is_key_pressed(ctx, Key::Space) {
            self.velocity.y = -JUMP_FORCE * y_direction;
        }
    }

    pub fn solve_collision_y(&mut self, rect: &Rect<f32, f32>) {
        let next_hbox = Rect::new(
            self.position.x,
            self.position.y + self.velocity.y,
            Self::PLAYER_SQUARE,
            Self::PLAYER_SQUARE,
        );
        if rect.collides_with_rect(next_hbox) {
            if self.velocity.y < 0. {
                self.position.y = rect.y + rect.h;
            }
            if self.velocity.y > 0. {
                self.position.y = rect.y - Self::PLAYER_SQUARE;
            }
            self.velocity.y = 0.;
        }
    }

    pub fn solve_collision_x(&mut self, rect: &Rect<f32, f32>) {
        let next_hbox = Rect::new(
            self.position.x + self.velocity.x,
            self.position.y,
            Self::PLAYER_SQUARE,
            Self::PLAYER_SQUARE,
        );
        if rect.collides_with_rect(next_hbox) {
            if self.velocity.x > 0. {
                self.position.x = rect.x - Self::PLAYER_SQUARE;
            }
            if self.velocity.x < 0. {
                self.position.x = rect.x + rect.w;
            }
            self.velocity.x = 0.;
        }
    }

    pub fn can_traverse_portal(&mut self, rect: &Rect<f32, f32>, axis: Axis) -> bool {
        if self.portal_traversed || !rect.collides_with_rect(self.get_hbox()) {
            return false;
        }
        let portal_center = rect.center();
        let player_to_entrance = (self.position + Self::HALF_SIZE) - portal_center;
        let player_to_exit = (self.position + Self::HALF_SIZE) + self.velocity - portal_center;
        match axis {
            Axis::Horizontal => {
                if player_to_entrance.x.signum() != player_to_exit.x.signum() {
                    self.portal_traversed = true;
                    return true;
                }
            }
            Axis::Vertical => {
                if player_to_entrance.y.signum() != player_to_exit.y.signum() {
                    self.portal_traversed = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn get_hbox(&self) -> Rect<f32, f32> {
        Rect::new(
            self.position.x,
            self.position.y,
            Self::PLAYER_SQUARE,
            Self::PLAYER_SQUARE,
        )
    }

    pub fn post_update(&mut self) {
        self.position += self.velocity;
    }
}
