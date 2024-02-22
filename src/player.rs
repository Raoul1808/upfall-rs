use tetra::{
    graphics::Rectangle,
    input::{self, Key},
    math::Vec2,
};

use crate::{
    tilemap::{Axis, Facing},
    world::WorldMode,
};

#[derive(Debug)]
pub struct Player {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
    portal_traversed: bool,
    can_jump: bool,
    fall_direction: f32,
    is_jumping: bool,
    flip_horizontal: bool,
    flip_vertical: bool,
    hit_spring: bool,
}

impl Player {
    pub const PLAYER_SQUARE: f32 = 16.;
    pub const HALF_SIZE: Vec2<f32> = Vec2::new(Self::PLAYER_SQUARE / 2., Self::PLAYER_SQUARE / 2.);
    pub const FALL_DOWN: f32 = 1.;
    pub const FALL_UP: f32 = -1.;
    pub const SPRING_FORCE: f32 = 10.;

    pub fn new(spawn_pos: Vec2<f32>) -> Player {
        Player {
            position: spawn_pos,
            velocity: Vec2::default(),
            portal_traversed: false,
            can_jump: false,
            fall_direction: 1.,
            is_jumping: false,
            flip_horizontal: false,
            flip_vertical: false,
            hit_spring: false,
        }
    }

    pub fn update(&mut self, ctx: &mut tetra::Context) {
        self.portal_traversed = false;
        const MAX_FALL_SPEED: f32 = 6.5;
        const GRAVITY: f32 = 0.3;
        const JUMP_FORCE: f32 = 5.5;
        const WALK_SPEED: f32 = 4.;
        const WALK_ACCELERATION: f32 = 0.75;

        let left = input::is_key_down(ctx, Key::Left);
        let right = input::is_key_down(ctx, Key::Right);
        let jump = input::is_key_pressed(ctx, Key::Space);
        let jumping = input::is_key_down(ctx, Key::Space);

        let mut target_speed = 0.;
        if left {
            target_speed = -WALK_SPEED;
        }
        if right {
            target_speed = WALK_SPEED;
        }
        if left == right {
            target_speed = 0.;
        }

        let direction = (target_speed - self.velocity.x).signum();
        self.velocity.x += WALK_ACCELERATION * direction;
        if (target_speed - self.velocity.x).signum() != direction {
            self.velocity.x = target_speed;
        }

        if self.hit_spring && self.velocity.y.signum() == self.fall_direction.signum() {
            self.velocity.y -= GRAVITY * self.fall_direction;
        } else {
            self.velocity.y += GRAVITY * self.fall_direction;
            if self.velocity.y.signum() == self.fall_direction.signum()
                && self.velocity.y.abs() > MAX_FALL_SPEED
            {
                self.velocity.y = MAX_FALL_SPEED * self.fall_direction;
            }
        }

        if self.hit_spring && self.velocity.y.abs() < MAX_FALL_SPEED {
            self.hit_spring = false;
        }

        if jump && self.can_jump {
            self.velocity.y = -JUMP_FORCE * self.fall_direction;
            self.can_jump = false;
            self.is_jumping = true;
        }

        if self.velocity.y == 0. || self.velocity.y.signum() == self.fall_direction.signum() {
            self.is_jumping = false;
        }

        if self.is_jumping && !jumping {
            self.is_jumping = false;
            self.velocity.y *= 0.33;
        }

        if self.velocity.x < 0. {
            self.flip_horizontal = true;
        }
        if self.velocity.x > 0. {
            self.flip_horizontal = false;
        }
    }

    pub fn on_world_change(&mut self, mode: WorldMode) {
        match mode {
            WorldMode::Dark => {
                self.fall_direction = Self::FALL_DOWN;
                self.flip_vertical = false;
            }
            WorldMode::Light => {
                self.fall_direction = Self::FALL_UP;
                self.flip_vertical = true;
            }
        }
    }

    fn on_land(&mut self) {
        self.can_jump = true;
    }

    pub fn on_spring(&mut self, facing: Facing) {
        if self.hit_spring {
            return;
        }
        match facing {
            Facing::Up => {
                self.velocity.y = -Self::SPRING_FORCE;
            }
            Facing::Down => {
                self.velocity.y = Self::SPRING_FORCE;
            }
            _ => {
                return;
            }
        }
        self.hit_spring = true;
        self.can_jump = true;
    }

    pub fn solve_collision_y(&mut self, rect: &Rectangle) {
        let next_hbox = Rectangle::new(
            self.position.x,
            self.position.y + self.velocity.y,
            Self::PLAYER_SQUARE,
            Self::PLAYER_SQUARE,
        );
        if rect.intersects(&next_hbox) {
            if self.velocity.y < 0. {
                // Top collision
                if self.fall_direction == Self::FALL_UP {
                    self.on_land();
                }
                self.position.y = rect.y + rect.height;
            }
            if self.velocity.y > 0. {
                if self.fall_direction == Self::FALL_DOWN {
                    self.on_land();
                }
                self.position.y = rect.y - Self::PLAYER_SQUARE;
            }
            self.velocity.y = 0.;
        }
    }

    pub fn solve_collision_x(&mut self, rect: &Rectangle) {
        let next_hbox = Rectangle::new(
            self.position.x + self.velocity.x,
            self.position.y,
            Self::PLAYER_SQUARE,
            Self::PLAYER_SQUARE,
        );
        if rect.intersects(&next_hbox) {
            if self.velocity.x > 0. {
                self.position.x = rect.x - Self::PLAYER_SQUARE;
            }
            if self.velocity.x < 0. {
                self.position.x = rect.x + rect.width;
            }
            self.velocity.x = 0.;
        }
    }

    pub fn can_traverse_portal(&mut self, rect: &Rectangle, axis: Axis) -> bool {
        if self.portal_traversed || !rect.intersects(&self.get_hbox()) {
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

    pub fn get_hbox(&self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y,
            Self::PLAYER_SQUARE,
            Self::PLAYER_SQUARE,
        )
    }

    pub fn post_update(&mut self) {
        self.position += self.velocity;
    }

    pub fn flip_horizontal(&self) -> bool {
        self.flip_horizontal
    }

    pub fn flip_vertical(&self) -> bool {
        self.flip_vertical
    }
}
