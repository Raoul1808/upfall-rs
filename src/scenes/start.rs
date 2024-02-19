use egui_tetra::egui;
use tetra::graphics::{self, Color};

use crate::level::LevelPack;

use super::{EditorScene, GameScene, Scene, Transition};

pub struct StartScene {
    editor: bool,
    quit: bool,
    play: bool,
    play_pack: bool,
    manage_packs: bool,
    packs: Vec<LevelPack>,
    selected_pack: usize,
}

impl StartScene {
    pub fn new() -> Self {
        Self {
            editor: false,
            quit: false,
            play: false,
            play_pack: false,
            manage_packs: false,
            packs: Vec::new(),
            selected_pack: 0,
        }
    }

    pub fn refresh_packs(&mut self) {
        if let Ok(packs) = LevelPack::get_packs_in_directory("levels") {
            self.packs = packs;
        }
    }
}

impl Scene for StartScene {
    fn update(
        &mut self,
        ctx: &mut tetra::Context,
        _egui_ctx: &egui_tetra::egui::CtxRef,
    ) -> tetra::Result<Transition> {
        if self.play_pack {
            self.play_pack = false;
            self.play = false;
            let pack = self.packs.remove(self.selected_pack);
            self.packs.clear();
            self.selected_pack = 0;
            return Ok(Transition::Push(Box::new(GameScene::with_pack(ctx, pack)?)));
        }
        if self.editor {
            self.editor = false;
            return Ok(Transition::Push(Box::new(EditorScene::new(ctx))));
        }
        if self.quit {
            return Ok(Transition::Pop);
        }
        Ok(Transition::None)
    }

    fn egui_layout(
        &mut self,
        _ctx: &mut tetra::Context,
        egui_ctx: &egui_tetra::egui::CtxRef,
    ) -> Result<(), egui_tetra::Error> {
        egui::Window::new("Upfall").show(egui_ctx, |ui| {
            ui.heading("PLACEHOLDER MENU");
            ui.label("This menu will not be present in the final 1.0 version of the game, as a proper menu system is still being worked on.");
            ui.separator();
            if ui.button("Play").clicked() {
                self.play = true;
                self.refresh_packs();
            }
            if ui.button("Packs").clicked() {
                self.manage_packs = true;
                self.refresh_packs();
            }
            if ui.button("Editor").clicked() {
                self.editor = true;
            }
            if ui.button("Quit").clicked() {
                self.quit = true;
            }
        });
        egui::Window::new("Play")
            .open(&mut self.play)
            .show(egui_ctx, |ui| {
                if self.packs.is_empty() {
                    ui.label("No packs detected :(");
                } else {
                    egui::ComboBox::from_label("Pack").show_index(
                        ui,
                        &mut self.selected_pack,
                        self.packs.len(),
                        |i| {
                            format!(
                                "[{}] {}",
                                if self.packs[i].is_zip { "ZIP" } else { "DIR" },
                                self.packs[i].name
                            )
                        },
                    );
                }
                if ui.button("Play pack").clicked() {
                    self.play_pack = true;
                }
            });
        egui::Window::new("Manage Packs")
            .open(&mut self.manage_packs)
            .show(egui_ctx, |ui| {
                if self.packs.is_empty() {
                    ui.label("No packs detected :(");
                } else {
                    egui::ComboBox::from_label("Pack").show_index(
                        ui,
                        &mut self.selected_pack,
                        self.packs.len(),
                        |i| {
                            format!(
                                "[{}] {}",
                                if self.packs[i].is_zip { "ZIP" } else { "DIR" },
                                self.packs[i].name
                            )
                        },
                    );
                    let current_pack = &self.packs[self.selected_pack];
                    if ui
                        .add_enabled(!current_pack.is_zip, egui::Button::new("Zip up"))
                        .clicked()
                    {
                        let res = LevelPack::make_zip_from_dir(&current_pack.location);
                        match res {
                            Ok(p) => self.packs.push(p),
                            Err(e) => println!("{:#?}", e),
                        }
                    }
                }
            });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context, _assets: &crate::Assets) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);
        Ok(())
    }
}
