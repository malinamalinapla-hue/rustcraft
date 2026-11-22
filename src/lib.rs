use azalea::prelude::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Arc, Mutex};

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

fn cfg_path() -> std::path::PathBuf {
    std::path::PathBuf::from("settings.json")
}
fn load() -> Settings {
    std::fs::read_to_string(cfg_path()).ok()
        .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
fn save(s: &Settings) { let _ = std::fs::write(cfg_path(), serde_json::to_string_pretty(s).unwrap()); }

#[derive(Resource)]
struct Ui {
    s: Settings,
    new_server: String,
    log: Vec<String>,
    rx: Option<Mutex<mpsc::Receiver<String>>>,
    petals: Vec<(f32, f32, f32)>,
    show_settings: bool,
}

#[derive(Default, Clone, Component)]
struct BotState { tx: Arc<Mutex<Option<mpsc::Sender<String>>>> }

async fn handle(_bot: Client, event: Event, st: BotState) -> anyhow::Result<()> {
    let say = |m: String| if let Some(tx) = st.tx.lock().unwrap().as_ref() { let _ = tx.send(m); };
    match event {
        Event::Login => say("Вход выполнен".into()),
        Event::Chat(m) => say(m.message().to_string()),
        Event::Disconnect(r) => say(format!("Отключено: {:?}", r)),
        _ => {}
    }
    Ok(())
}

fn connect(addr: String, nick: String) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        rt.block_on(async move {
            let st = BotState::default();
            *st.tx.lock().unwrap() = Some(tx.clone());
            let acc = Account::offline(&nick);
            let r = ClientBuilder::new().set_handler(handle).set_state(st).start(acc, addr.as_str()).await;
            let _ = tx.send(format!("Соединение закрыто: {:?}", r.err()));
        });
    });
    rx
}

#[bevy_main]
pub fn run() {
    let petals = (0..40).map(|i| ((i * 37 % 100) as f32 / 100.0, (i * 61 % 100) as f32 / 100.0, 0.3 + (i % 5) as f32 * 0.15)).collect();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::srgb(0.10, 0.07, 0.18)))
        .insert_resource(Ui { s: load(), new_server: String::new(), log: vec![], rx: None, petals, show_settings: false })
        .add_systems(Startup, |mut c: Commands| { c.spawn(Camera2dBundle::default()); })
        .add_systems(Update, launcher)
        .run();
}

fn launcher(mut ctx: EguiContexts, mut ui: ResMut<Ui>, time: Res<Time>) {
    let t = time.elapsed_seconds();
    if let Some(rx) = &ui.rx {
        let msgs: Vec<String> = rx.lock().unwrap().try_iter().collect();
        ui.log.extend(msgs);
        if ui.log.len() > 200 { let d = ui.log.len() - 200; ui.log.drain(0..d); }
    }
    let [r, g, b] = ui.s.accent;
    let accent = egui::Color32::from_rgb(r, g, b);
    let c = ctx.ctx_mut();

    let screen = c.screen_rect();
    let p = c.layer_painter(egui::LayerId::background());
    p.rect_filled(screen, 0.0, egui::Color32::from_rgb(28, 18, 52));
    p.circle_filled(screen.center_top() + egui::vec2(0.0, 120.0), 160.0, egui::Color32::from_rgba_unmultiplied(255, 140, 190, 40));
    for (x, y, sp) in ui.petals.iter() {
        let yy = (y + t * 0.05 * sp) % 1.0;
        let xx = x + (t * sp + y * 6.0).sin() * 0.02;
        p.circle_filled(egui::pos2(screen.left() + xx * screen.width(), screen.top() + yy * screen.height()), 3.0 + sp * 3.0, accent.gamma_multiply(0.7));
    }
    c.request_repaint();

    let panel = egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(20, 14, 40, ui.s.panel_alpha))
        .rounding(ui.s.rounding).inner_margin(16.0)
        .stroke(egui::Stroke::new(1.5, accent));

    egui::Window::new("RustCraft").frame(panel).collapsible(false).resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(c, |u| {
            u.heading(egui::RichText::new("✿ RustCraft ✿").color(accent));
            u.horizontal(|u| { u.label("Ник:"); u.text_edit_singleline(&mut ui.s.nick); });
            let n = ui.s.servers.len();
            egui::ComboBox::from_label("Сервер").selected_text(ui.s.servers.get(ui.s.selected).cloned().unwrap_or_default())
                .show_ui(u, |u| { for i in 0..n { let s = ui.s.servers[i].clone(); u.selectable_value(&mut ui.s.selected, i, s); } });
            u.horizontal(|u| {
                u.text_edit_singleline(&mut ui.new_server);
                if u.button("+").clicked() && !ui.new_server.is_empty() { let s = std::mem::take(&mut ui.new_server); ui.s.servers.push(s); }
                if u.button("🗑").clicked() && n > 1 { let i = ui.s.selected; ui.s.servers.remove(i); ui.s.selected = 0; }
            });
            u.label("Версия: 1.21.8 (офлайн-режим)");
            if u.add_sized([220.0, 44.0], egui::Button::new(egui::RichText::new("▶ ИГРАТЬ").size(22.0)).fill(accent)).clicked() {
                save(&ui.s);
                let addr = ui.s.servers[ui.s.selected].clone();
                ui.log.push(format!("Подключаюсь к {addr}…"));
                ui.rx = Some(Mutex::new(connect(addr, ui.s.nick.clone())));
            }
            if u.button("⚙ Настройки").clicked() { ui.show_settings = !ui.show_settings; }
            if ui.show_settings {
                u.separator();
                u.horizontal(|u| { u.label("Цвет:"); u.color_edit_button_srgb(&mut ui.s.accent); });
                u.add(egui::Slider::new(&mut ui.s.panel_alpha, 30..=255).text("прозрачность"));
                u.add(egui::Slider::new(&mut ui.s.rounding, 0.0..=32.0).text("скругление"));
                if u.button("Сохранить").clicked() { save(&ui.s); }
            }
            u.separator();
            egui::ScrollArea::vertical().max_height(120.0).stick_to_bottom(true).show(u, |u| { for l in &ui.log { u.label(l); } });
        });
                 }
