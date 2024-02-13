use egui_tetra::egui::{self, CtxRef};
use tetra::{
    graphics::{self, Camera, Color, DrawParams},
    input::{self, Key},
    math::Vec2,
    Event,
};

use crate::{
    level::Level,
    palette::Palette,
    tilemap::{Axis, Facing, Tile, Tilemap},
    world::WorldMode,
    Assets,
};

use super::{GameScene, Scene, Transition};

fn color_egui(ui: &mut egui::Ui, label: &str, color: &mut Color) {
    let mut col_bytes = [color.r, color.g, color.b];
    ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_rgb(&mut col_bytes);
    });
    color.r = col_bytes[0];
    color.g = col_bytes[1];
    color.b = col_bytes[2];
}

pub struct EditorScene {
    dark_tilemap: Tilemap,
    light_tilemap: Tilemap,
    mode: WorldMode,
    spawn_pos: Vec2<f32>,
    mouse_pos: Vec2<f32>,
    facing: Facing,
    axis: Axis,
    tile: Tile,
    tilemap_size: Vec2<usize>,
    camera: Camera,
    palette: Palette,
    level_name: String,
    level_author: String,
    quit: bool,
}

impl EditorScene {
    pub const TILEMAP_MIN_X: usize = 40;
    pub const TILEMAP_MIN_Y: usize = 23;
    pub const TILEMAP_MAX_X: usize = 1000;
    pub const TILEMAP_MAX_Y: usize = 1000;
    pub const ZOOM_MIN: f32 = 1.0;
    pub const ZOOM_MAX: f32 = 8.0;
    pub const DEFAULT_LEVEL_NAME: &'static str = "Untitled Level";
    pub const DEFAULT_AUTHOR_NAME: &'static str = "Unnamed Mapmaker";

    pub fn new(ctx: &mut tetra::Context) -> EditorScene {
        let tilemap_size = (80, 45);
        let mut camera = Camera::with_window_size(ctx);
        camera.position = camera.visible_rect().bottom_right();
        camera.update();
        EditorScene {
            dark_tilemap: Tilemap::new(tilemap_size, (16., 16.)),
            light_tilemap: Tilemap::new(tilemap_size, (16., 16.)),
            mode: WorldMode::Dark,
            spawn_pos: Vec2::default(),
            mouse_pos: Vec2::default(),
            facing: Facing::Up,
            axis: Axis::Horizontal,
            tile: Tile::Solid,
            tilemap_size: tilemap_size.into(),
            camera,
            palette: Palette::Simple {
                dark: Color::BLACK,
                light: Color::WHITE,
            },
            level_name: Self::DEFAULT_LEVEL_NAME.to_string(),
            level_author: Self::DEFAULT_AUTHOR_NAME.to_string(),
            quit: false,
        }
    }

    fn keyboard_update(&mut self, ctx: &mut tetra::Context) {
        let ctrl =
            input::is_key_down(ctx, Key::LeftCtrl) || input::is_key_down(ctx, Key::RightCtrl);
        let shift =
            input::is_key_down(ctx, Key::LeftShift) || input::is_key_down(ctx, Key::RightShift);
        
        if input::is_key_pressed(ctx, Key::Num1) {
            self.tile = Tile::Solid;
        }
        if input::is_key_pressed(ctx, Key::Num2) {
            self.tile = Tile::Spike(self.facing);
        }
        if input::is_key_pressed(ctx, Key::Num3) {
            self.tile = Tile::Portal(self.axis);
        }

        if input::is_key_pressed(ctx, Key::Up) {
            self.axis = Axis::Vertical;
            self.facing = Facing::Up;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }
        if input::is_key_pressed(ctx, Key::Down) {
            self.axis = Axis::Vertical;
            self.facing = Facing::Down;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }
        if input::is_key_pressed(ctx, Key::Left) {
            self.axis = Axis::Horizontal;
            self.facing = Facing::Left;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }
        if input::is_key_pressed(ctx, Key::Right) {
            self.axis = Axis::Horizontal;
            self.facing = Facing::Right;
            self.tile.set_facing(self.facing);
            self.tile.set_axis(self.axis);
        }

        if input::is_key_pressed(ctx, Key::T) {
            self.mode.switch();
        }

        const CAMERA_MOVE: f32 = 5.;
        if input::is_key_down(ctx, Key::A) {
            self.camera.position.x -= CAMERA_MOVE;
        }
        if input::is_key_down(ctx, Key::D) {
            self.camera.position.x += CAMERA_MOVE;
        }
        if input::is_key_down(ctx, Key::W) {
            self.camera.position.y -= CAMERA_MOVE;
        }
        if input::is_key_down(ctx, Key::S) {
            self.camera.position.y += CAMERA_MOVE;
        }

        if ctrl && !shift {
            if input::is_key_pressed(ctx, Key::S) {
                self.save_level();
            }

            if input::is_key_pressed(ctx, Key::O) {
                self.load_level();
            }
        }
    }

    fn mouse_update(&mut self, ctx: &mut tetra::Context) {
        let ctrl =
            input::is_key_down(ctx, Key::LeftCtrl) || input::is_key_down(ctx, Key::RightCtrl);
        let shift =
            input::is_key_down(ctx, Key::LeftShift) || input::is_key_down(ctx, Key::RightShift);

        let tilemap = match self.mode {
            WorldMode::Dark => &mut self.dark_tilemap,
            WorldMode::Light => &mut self.light_tilemap,
        };

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            tilemap.set_tile_f32(self.mouse_pos, self.tile);
        }

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            tilemap.set_tile_f32(self.mouse_pos, Tile::None);
        }

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Middle) {
            self.spawn_pos = tilemap.snap(self.mouse_pos);
        }
    }

    fn save_level(&self) {
        let level = Level {
            name: self.level_name.clone(),
            author: self.level_author.clone(),
            dark_tilemap: self.dark_tilemap.clone(),
            light_tilemap: self.light_tilemap.clone(),
            spawn_pos: self.spawn_pos,
            palette: self.palette,
        };
        match level.save("level.umdx") {
            Ok(_) => {}
            Err(e) => println!("{:?}", e),
        }
    }

    fn load_level(&mut self) {
        match Level::load("level.umdx") {
            Ok(l) => {
                self.dark_tilemap = l.dark_tilemap;
                self.light_tilemap = l.light_tilemap;
                self.spawn_pos = l.spawn_pos;
                self.palette = l.palette;
                self.level_name = l.name;
                self.level_author = l.author;
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

impl Scene for EditorScene {
    fn event(&mut self, _ctx: &mut tetra::Context, event: tetra::Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.camera.set_viewport_size(width as f32, height as f32);
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut tetra::Context, egui_ctx: &CtxRef) -> tetra::Result<Transition> {
        if self.quit {
            return Ok(Transition::Pop);
        }
        self.mouse_pos = self.camera.mouse_position(ctx);

        let wants_keyboard = egui_ctx.wants_keyboard_input();
        let wants_mouse = egui_ctx.wants_pointer_input();

        if !wants_keyboard {
            self.keyboard_update(ctx);
        }

        if !wants_mouse {
            self.mouse_update(ctx)
        }

        if !wants_keyboard && !wants_mouse && input::is_key_pressed(ctx, Key::Enter) {
            let level = Level {
                name: self.level_name.clone(),
                author: self.level_author.clone(),
                dark_tilemap: self.dark_tilemap.clone(),
                light_tilemap: self.light_tilemap.clone(),
                spawn_pos: self.spawn_pos,
                palette: self.palette,
            };
            return Ok(Transition::Push(Box::new(GameScene::new(ctx, level)?)));
        }

        self.camera.update();
        Ok(Transition::None)
    }

    fn egui_layout(
        &mut self,
        _ctx: &mut tetra::Context,
        egui_ctx: &egui_tetra::egui::CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        egui::Window::new("Toolbox and Properties").show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load Level").clicked() {
                    self.load_level();
                }
                if ui.button("Save Level").clicked() {
                    self.save_level();
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Level Name");
                ui.text_edit_singleline(&mut self.level_name);
            });
            ui.horizontal(|ui| {
                ui.label("Levle Author");
                ui.text_edit_singleline(&mut self.level_author);
            });
            ui.horizontal(|ui| {
                ui.label("Tilemap Size");
                ui.add(
                    egui::DragValue::new(&mut self.tilemap_size.x)
                        .speed(0.1)
                        .clamp_range(Self::TILEMAP_MIN_X..=Self::TILEMAP_MAX_X),
                );
                ui.add(
                    egui::DragValue::new(&mut self.tilemap_size.y)
                        .speed(0.1)
                        .clamp_range(Self::TILEMAP_MIN_Y..=Self::TILEMAP_MAX_Y),
                );
                self.dark_tilemap.resize(self.tilemap_size);
                self.light_tilemap.resize(self.tilemap_size);
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Zoom Level");
                let mut zoom_scale = self.camera.scale.x;
                ui.add(egui::Slider::new(
                    &mut zoom_scale,
                    Self::ZOOM_MIN..=Self::ZOOM_MAX,
                ));
                self.camera.scale.x = zoom_scale;
                self.camera.scale.y = zoom_scale;
                self.camera.update();
            });
            ui.horizontal(|ui| {
                ui.label(format!("Current tilemap mode: {}", self.mode));
                if ui.button("Switch").clicked() {
                    self.mode.switch();
                }
            });
            egui::ComboBox::from_label("Place Tile")
                .selected_text(self.tile.type_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.tile, Tile::Solid, "Solid");
                    ui.selectable_value(&mut self.tile, Tile::Spike(self.facing), "Spike");
                    ui.selectable_value(&mut self.tile, Tile::Portal(self.axis), "Portal");
                });
            if let Tile::Spike(ref mut facing) = self.tile {
                egui::ComboBox::from_label("Facing")
                    .selected_text(self.facing.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.facing, Facing::Up, "Up");
                        ui.selectable_value(&mut self.facing, Facing::Down, "Down");
                        ui.selectable_value(&mut self.facing, Facing::Left, "Left");
                        ui.selectable_value(&mut self.facing, Facing::Right, "Right");
                    });
                *facing = self.facing;
            }
            if let Tile::Portal(ref mut axis) = self.tile {
                egui::ComboBox::from_label("Axis")
                    .selected_text(self.axis.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.axis, Axis::Horizontal, "Horizontal");
                        ui.selectable_value(&mut self.axis, Axis::Vertical, "Vertical");
                    });
                *axis = self.axis;
            }
            ui.separator();
            egui::ComboBox::from_label("Palette Type")
                .selected_text(self.palette.type_str())
                .show_ui(ui, |ui| {
                    for palette in Palette::default_all() {
                        ui.selectable_value(&mut self.palette, palette, palette.type_str());
                    }
                });

            match self.palette {
                Palette::Simple {
                    ref mut dark,
                    ref mut light,
                } => {
                    color_egui(ui, "Dark Color", dark);
                    color_egui(ui, "Light Color", light);
                }
                Palette::Lerp {
                    ref mut dark1,
                    ref mut dark2,
                    ref mut light1,
                    ref mut light2,
                } => {
                    color_egui(ui, "First Dark Color", dark1);
                    color_egui(ui, "Second Dark Color", dark2);
                    color_egui(ui, "First Light Color", light1);
                    color_egui(ui, "Second Light Color", light2);
                }
                _ => {}
            }
            ui.separator();
            if ui.button("Quit Editor").clicked() {
                self.quit = true;
            }
        });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context, assets: &Assets) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);
        graphics::set_transform_matrix(ctx, self.camera.as_matrix());
        let tilemap_rect = self.dark_tilemap.rect();
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(tilemap_rect.top_left())
                .scale(tilemap_rect.bottom_right())
                .color(Color::rgb8(100, 149, 237)),
        );
        let (dark_alpha, light_alpha) = match self.mode {
            WorldMode::Dark => (1., 0.33),
            WorldMode::Light => (0.33, 1.),
        };
        self.dark_tilemap
            .render_tilemap(ctx, assets, Color::WHITE.with_alpha(dark_alpha));
        self.light_tilemap
            .render_tilemap(ctx, assets, Color::BLACK.with_alpha(light_alpha));
        assets
            .player
            .draw(ctx, DrawParams::new().position(self.spawn_pos));
        if self.dark_tilemap.rect().contains_point(self.mouse_pos) {
            assets.pixel.draw(
                ctx,
                DrawParams::new()
                    .position(self.dark_tilemap.snap(self.mouse_pos))
                    .scale(self.dark_tilemap.tile_size())
                    .color(Color::WHITE.with_alpha(1. / 3.)),
            );
        }
        graphics::reset_transform_matrix(ctx);
        Ok(())
    }
}
