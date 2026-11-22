use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Settings {
    nick: String,
    servers: Vec<String>,
    selected: usize,
    accent: [u8; 3],
    panel_alpha: u8,
    rounding: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            nick: "Player".into(),
            servers: vec!["localhost:25565".into()],
            selected: 0,
            accent: [255, 143, 190],
            panel_alpha: 170,
            rounding: 16.0,
        }
    }
}

fn load() -> Settings {
    std::fs::read_to_string("settings.json").ok()
        .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
fn save(s: &Settings) {
    if let Ok(j) = serde_json::to_string_pretty(s) { let _ = std::fs::write("settings.json", j); }
}

#[derive(Resource)]
struct Ui {
    s: Settings,
    new_server: String,
    log: Vec<String>,
    show_settings: bool,
}

#[bevy_main]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::srgb(0.10, 0.07, 0.18)))
        .insert_resource(Ui { s: load(), new_server: String::new(), log: vec![], show_settings: false })
        .add_systems(Startup, setup)
        .add_systems(Update, launcher)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn launcher(mut ctx: EguiContexts, mut ui: ResMut<Ui>, time: Res<Time>) {
    let t = time.elapsed_seconds();
    let [r, g, b] = ui.s.accent;
    let accent = egui::Color32::from_rgb(r, g, b);
    let alpha = ui.s.panel_alpha;
    let rounding = ui.s.rounding;
    let c = ctx.ctx_mut();

    let screen = c.screen_rect();
    let p = c.layer_painter(egui::LayerId::background());
    p.rect_filled(screen, 0.0, egui::Color32::from_rgb(28, 18, 52));
    p.circle_filled(
        screen.center_top() + egui::vec2(0.0, 120.0),
        160.0,
        egui::Color32::from_rgba_unmultiplied(255, 140, 190, 40),
    );
    for i in 0..40 {
        let x = (i * 37 % 100) as f32 / 100.0;
        let y = (i * 61 % 100) as f32 / 100.0;
        let sp = 0.3 + (i % 5) as f32 * 0.15;
        let yy = (y + t * 0.05 * sp) % 1.0;
        let xx = x + (t * sp + y * 6.0).sin() * 0.02;
        p.circle_filled(
            egui::pos2(screen.left() + xx * screen.width(), screen.top() + yy * screen.height()),
            3.0 + sp * 3.0,
            accent.gamma_multiply(0.7),
        );
    }
    c.request_repaint();

    let panel = egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(20, 14, 40, alpha))
        .rounding(rounding)
        .inner_margin(16.0)
        .stroke(egui::Stroke::new(1.5, accent));

    let ui = &mut *ui;
    egui::Window::new("RustCraft")
        .frame(panel)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(c, |u| {
            u.heading(egui::RichText::new("✿ RustCraft ✿").color(accent));
            u.horizontal(|u| {
                u.label("Ник:");
                u.text_edit_singleline(&mut ui.s.nick);
            });
            let n = ui.s.servers.len();
            let current = ui.s.servers.get(ui.s.selected).cloned().unwrap_or_default();
            egui::ComboBox::from_label("Сервер").selected_text(current).show_ui(u, |u| {
                for i in 0..n {
                    let name = ui.s.servers[i].clone();
                    u.selectable_value(&mut ui.s.selected, i, name);
                }
            });
            u.horizontal(|u| {
                u.text_edit_singleline(&mut ui.new_server);
                if u.button("+").clicked() && !ui.new_server.is_empty() {
                    let s = std::mem::take(&mut ui.new_server);
                    ui.s.servers.push(s);
                }
                if u.button("🗑").clicked() && n > 1 {
                    let i = ui.s.selected;
                    ui.s.servers.remove(i);
                    ui.s.selected = 0;
                }
            });
            u.label("Версия: 1.21.8 (офлайн-режим)");
            let play = egui::Button::new(egui::RichText::new("▶ ИГРАТЬ").size(22.0)).fill(accent);
            if u.add_sized([220.0, 44.0], play).clicked() {
                save(&ui.s);
                ui.log.push("Подключение к серверу будет в следующем обновлении".to_string());
            }
            if u.button("⚙ Настройки").clicked() {
                ui.show_settings = !ui.show_settings;
            }
            if ui.show_settings {
                u.separator();
                u.horizontal(|u| {
                    u.label("Цвет:");
                    u.color_edit_button_srgb(&mut ui.s.accent);
                });
                u.add(egui::Slider::new(&mut ui.s.panel_alpha, 30..=255).text("прозрачность"));
                u.add(egui::Slider::new(&mut ui.s.rounding, 0.0..=32.0).text("скругление"));
                if u.button("Сохранить").clicked() {
                    save(&ui.s);
                }
            }
            u.separator();
            egui::ScrollArea::vertical().max_height(120.0).show(u, |u| {
                for l in &ui.log {
                    u.label(l);
                }
            });
        });
              }
