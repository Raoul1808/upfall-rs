use std::path::PathBuf;

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
    level: Level,
    world_mode: WorldMode,
    mouse_pos: Vec2<f32>,
    facing: Facing,
    axis: Axis,
    tile: Tile,
    camera: Camera,
    quit: bool,
    level_path: Option<PathBuf>,
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
    pub const DEFAULT_TILEMAP_SIZE: (usize, usize) = (80, 45);
    pub const DEFAULT_TILE_SIZE: (f32, f32) = (16., 16.);

    pub fn new(ctx: &mut tetra::Context) -> EditorScene {
        let mut camera = Camera::with_window_size(ctx);
        camera.position = camera.visible_rect().bottom_right();
        camera.update();
        EditorScene {
            level: Self::default_level(),
            world_mode: WorldMode::Dark,
            mouse_pos: Vec2::default(),
            facing: Facing::Up,
            axis: Axis::Horizontal,
            tile: Tile::Solid,
            camera,
            quit: false,
            level_path: None,
        }
    }

    pub fn default_level() -> Level {
        Level {
            name: Self::DEFAULT_LEVEL_NAME.to_string(),
            author: Self::DEFAULT_AUTHOR_NAME.to_string(),
            dark_tilemap: Tilemap::new(Self::DEFAULT_TILEMAP_SIZE, Self::DEFAULT_TILE_SIZE),
            light_tilemap: Tilemap::new(Self::DEFAULT_TILEMAP_SIZE, Self::DEFAULT_TILE_SIZE),
            palette: Palette::default(),
            spawn_pos: Vec2::zero(),
            end_pos: Vec2::zero(),
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
        if input::is_key_pressed(ctx, Key::Num4) {
            self.tile = Tile::Key;
        }
        if input::is_key_pressed(ctx, Key::Num5) {
            self.tile = Tile::Spring(self.facing);
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
            self.world_mode.switch();
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

        let tilemap = match self.world_mode {
            WorldMode::Dark => &mut self.level.dark_tilemap,
            WorldMode::Light => &mut self.level.light_tilemap,
        };

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Left) {
            tilemap.set_tile_f32(self.mouse_pos, self.tile);
        }

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Right) {
            tilemap.set_tile_f32(self.mouse_pos, Tile::None);
        }

        if !shift && !ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Middle) {
            self.level.spawn_pos = tilemap.snap(self.mouse_pos);
        }

        if !shift && ctrl && input::is_mouse_button_down(ctx, input::MouseButton::Middle) {
            self.level.end_pos = tilemap.snap(self.mouse_pos);
        }
    }

    fn new_level(&mut self) {
        self.level = Self::default_level();
        self.level_path = None;
    }

    fn save_level(&mut self) {
        match &self.level_path {
            Some(p) => match self.level.save_file(p) {
                Ok(_) => {}
                Err(e) => println!("Error saving level at {}: {:?}", p.display(), e),
            },
            None => self.save_level_as(),
        }
    }

    fn save_level_as(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Upfall-RS Map Data", &["umdx"])
            .save_file();
        if let Some(mut p) = path {
            p.set_extension("umdx");
            self.level_path = Some(p.clone());
            match self.level.save_file(&p) {
                Ok(_) => {}
                Err(e) => println!("Error saving level at {}: {:?}", p.display(), e),
            }
        }
    }

    fn load_level(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Upfall-RS Map Data", &["umdx"])
            .pick_file();
        if let Some(file) = file {
            match Level::load_file(&file) {
                Ok(l) => {
                    self.level = l;
                    self.level_path = Some(file);
                }
                Err(e) => println!("Error loading level at {}: {:?}", file.display(), e),
            }
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
            return Ok(Transition::Push(Box::new(GameScene::new(
                ctx,
                self.level.clone(),
            )?)));
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
            ui.label(format!("Current level: {:?}", self.level_path));
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    self.new_level();
                }
                if ui.button("Open").clicked() {
                    self.load_level();
                }
                if ui.button("Save").clicked() {
                    self.save_level();
                }
                if ui.button("Save As").clicked() {
                    self.save_level_as();
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Level Name");
                ui.text_edit_singleline(&mut self.level.name);
            });
            ui.horizontal(|ui| {
                ui.label("Level Author");
                ui.text_edit_singleline(&mut self.level.author);
            });
            ui.horizontal(|ui| {
                ui.label("Tilemap Size");
                let mut tilemap_size = self.level.dark_tilemap.size();
                ui.add(
                    egui::DragValue::new(&mut tilemap_size.x)
                        .speed(0.1)
                        .clamp_range(Self::TILEMAP_MIN_X..=Self::TILEMAP_MAX_X),
                );
                ui.add(
                    egui::DragValue::new(&mut tilemap_size.y)
                        .speed(0.1)
                        .clamp_range(Self::TILEMAP_MIN_Y..=Self::TILEMAP_MAX_Y),
                );
                self.level.dark_tilemap.resize(tilemap_size);
                self.level.light_tilemap.resize(tilemap_size);
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
                ui.label(format!("Current tilemap mode: {}", self.world_mode));
                if ui.button("Switch").clicked() {
                    self.world_mode.switch();
                }
            });
            egui::ComboBox::from_label("Place Tile")
                .selected_text(self.tile.type_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.tile, Tile::Solid, "Solid");
                    ui.selectable_value(&mut self.tile, Tile::Spike(self.facing), "Spike");
                    ui.selectable_value(&mut self.tile, Tile::Portal(self.axis), "Portal");
                    ui.selectable_value(&mut self.tile, Tile::Key, "Key");
                    ui.selectable_value(&mut self.tile, Tile::Spring(self.facing), "Spring");
                });
            match self.tile {
                Tile::Spike(ref mut facing) => {
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
                Tile::Portal(ref mut axis) => {
                    egui::ComboBox::from_label("Axis")
                        .selected_text(self.axis.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.axis, Axis::Horizontal, "Horizontal");
                            ui.selectable_value(&mut self.axis, Axis::Vertical, "Vertical");
                        });
                    *axis = self.axis;
                }
                Tile::Spring(ref mut facing) => {
                    egui::ComboBox::from_label("Facing")
                        .selected_text(self.facing.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.facing, Facing::Up, "Up");
                            ui.selectable_value(&mut self.facing, Facing::Down, "Down");
                        });
                    *facing = self.facing;
                }
                _ => {}
            }
            ui.separator();
            egui::ComboBox::from_label("Palette Type")
                .selected_text(self.level.palette.type_str())
                .show_ui(ui, |ui| {
                    for palette in Palette::default_all() {
                        ui.selectable_value(&mut self.level.palette, palette, palette.type_str());
                    }
                });

            match self.level.palette {
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
        let tilemap_rect = self.level.dark_tilemap.rect();
        assets.pixel.draw(
            ctx,
            DrawParams::new()
                .position(tilemap_rect.top_left())
                .scale(tilemap_rect.bottom_right())
                .color(Color::rgb8(100, 149, 237)),
        );
        let (dark_alpha, light_alpha) = match self.world_mode {
            WorldMode::Dark => (1., 0.33),
            WorldMode::Light => (0.33, 1.),
        };
        self.level
            .dark_tilemap
            .render_tilemap(ctx, assets, Color::WHITE.with_alpha(dark_alpha));
        self.level
            .light_tilemap
            .render_tilemap(ctx, assets, Color::BLACK.with_alpha(light_alpha));
        assets.player.draw(ctx, self.level.spawn_pos);
        assets.door.draw(ctx, self.level.end_pos);
        if self
            .level
            .dark_tilemap
            .rect()
            .contains_point(self.mouse_pos)
        {
            assets.pixel.draw(
                ctx,
                DrawParams::new()
                    .position(self.level.dark_tilemap.snap(self.mouse_pos))
                    .scale(self.level.dark_tilemap.tile_size())
                    .color(Color::WHITE.with_alpha(1. / 3.)),
            );
        }
        graphics::reset_transform_matrix(ctx);
        Ok(())
    }
}
