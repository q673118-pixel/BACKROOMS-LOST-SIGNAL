use bevy::{
    input::touch::Touches,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{observe, Activate, Button as UiButton},
    window::{MonitorSelection, PrimaryWindow, WindowMode},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "BACKROOMS: LOST SIGNAL".into(),
                resizable: false,
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<UiModel>()
        .init_resource::<UiCommand>()
        .init_resource::<Settings>()
        .init_resource::<TouchState>()
        .init_resource::<CameraLook>()
        .add_systems(Startup, (load_settings, setup_camera_and_light))
        .add_systems(
            Update,
            (
                apply_ui_command,
                rebuild_ui,
                sync_world,
                mobile_controls,
                update_vhs_tint,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
enum UiPage {
    #[default]
    Menu,
    Settings,
    DebugWorld,
    Playing,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
enum SettingsTab {
    #[default]
    Controls,
    Vhs,
    Graphics,
    Audio,
    Interface,
}

#[derive(Resource)]
struct UiModel {
    page: UiPage,
    tab: SettingsTab,
    revision: u64,
}
impl Default for UiModel {
    fn default() -> Self {
        Self { page: UiPage::Menu, tab: SettingsTab::Controls, revision: 0 }
    }
}

#[derive(Clone, Copy)]
enum SettingKey {
    Sensitivity, JoystickSize, JoystickOpacity, InvertY,
    VhsEnabled, VhsIntensity, Scanlines, Noise, Chromatic, Distortion, Flicker,
    RenderDistance, Shadows, FpsLimit,
    MasterVolume, EnvironmentVolume, EffectsVolume, MusicVolume,
    UiScale, ShowFps,
}

#[derive(Clone, Copy)]
enum UiAction {
    Page(UiPage),
    Tab(SettingsTab),
    Adjust(SettingKey, f32),
    Toggle(SettingKey),
    Reset,
}

#[derive(Resource, Default)]
struct UiCommand(Option<UiAction>);

#[derive(Resource, Serialize, Deserialize, Clone)]
#[serde(default)]
struct Settings {
    sensitivity: f32,
    joystick_size: f32,
    joystick_opacity: f32,
    invert_y: bool,
    vhs_enabled: bool,
    vhs_intensity: f32,
    scanlines: f32,
    noise: f32,
    chromatic: f32,
    distortion: f32,
    flicker: f32,
    render_distance: f32,
    shadows: bool,
    fps_limit: u32,
    master_volume: f32,
    environment_volume: f32,
    effects_volume: f32,
    music_volume: f32,
    ui_scale: f32,
    show_fps: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 1.0, joystick_size: 1.0, joystick_opacity: 0.65, invert_y: false,
            vhs_enabled: true, vhs_intensity: 0.35, scanlines: 0.45, noise: 0.25,
            chromatic: 0.15, distortion: 0.10, flicker: 0.08,
            render_distance: 45.0, shadows: false, fps_limit: 60,
            master_volume: 0.8, environment_volume: 0.8, effects_volume: 0.8,
            music_volume: 0.5, ui_scale: 1.0, show_fps: false,
        }
    }
}
impl Settings {
    fn file_path() -> PathBuf {
        let base = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        base.join("backrooms_lost_signal_settings.json")
    }
    fn load() -> Self {
        fs::read_to_string(Self::file_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::file_path(), json);
        }
    }
}

#[derive(Component)]
struct UiRoot;
#[derive(Component)]
struct WorldEntity;
#[derive(Component)]
struct Collider {
    center: Vec3,
    half: Vec3,
}
#[derive(Component)]
struct VhsTint;

#[derive(Resource, Default)]
struct TouchState {
    left_id: Option<u64>,
    left_origin: Vec2,
    axis: Vec2,
}
#[derive(Resource, Default)]
struct CameraLook {
    yaw: f32,
    pitch: f32,
}

fn load_settings(mut settings: ResMut<Settings>) {
    *settings = Settings::load();
}

fn setup_camera_and_light(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.65, 5.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
        #[cfg(target_os = "android")]
        Msaa::Off,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight { intensity: 500_000.0, range: 18.0, shadows_enabled: false, ..default() },
        Transform::from_xyz(0.0, 3.0, -3.0),
    ));
}

fn text_style(size: f32) -> TextFont {
    TextFont { font_size: FontSize::Px(size), ..default() }
}

fn button_bundle(label: String, action: UiAction) -> impl Bundle {
    (
        UiButton,
        Hovered::default(),
        Node {
            width: px(360),
            min_height: px(54),
            margin: UiRect::vertical(px(5)),
            padding: UiRect::axes(px(16), px(10)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.49, 0.43, 0.27)),
        BackgroundColor(Color::srgba(0.13, 0.13, 0.10, 0.94)),
        children![(
            Text::new(label),
            text_style(20.0),
            TextColor(Color::srgb(0.86, 0.81, 0.66)),
        )],
        observe(move |_: On<Activate>, mut command: ResMut<UiCommand>| {
            command.0 = Some(action);
        }),
    )
}

fn label_bundle(label: String, size: f32) -> impl Bundle {
    (Text::new(label), text_style(size), TextColor(Color::srgb(0.82, 0.78, 0.65)))
}

fn spawn_ui(commands: &mut Commands, model: &UiModel, settings: &Settings) {
    let root = commands.spawn((
        UiRoot,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
            padding: UiRect::all(px(18)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.035, 0.025, 0.78)),
    )).id();

    commands.entity(root).with_children(|p| {
        match model.page {
            UiPage::Menu => {
                p.spawn(label_bundle("BACKROOMS".into(), 54.0));
                p.spawn(label_bundle("L O S T   S I G N A L".into(), 19.0));
                p.spawn(label_bundle("AN ATMOSPHERIC HORROR PROTOTYPE".into(), 12.0));
                p.spawn(button_bundle("ИГРАТЬ".into(), UiAction::Page(UiPage::Playing)));
                p.spawn(button_bundle("ТЕСТ — DEBUG WORLD".into(), UiAction::Page(UiPage::DebugWorld)));
                p.spawn(button_bundle("НАСТРОЙКИ".into(), UiAction::Page(UiPage::Settings)));
            }
            UiPage::Settings => {
                p.spawn(label_bundle("НАСТРОЙКИ".into(), 38.0));
                p.spawn(label_bundle("Выбери категорию".into(), 15.0));
                p.spawn(button_bundle("УПРАВЛЕНИЕ".into(), UiAction::Tab(SettingsTab::Controls)));
                p.spawn(button_bundle("VHS-ФИЛЬТР".into(), UiAction::Tab(SettingsTab::Vhs)));
                p.spawn(button_bundle("ГРАФИКА".into(), UiAction::Tab(SettingsTab::Graphics)));
                p.spawn(button_bundle("ЗВУК".into(), UiAction::Tab(SettingsTab::Audio)));
                p.spawn(button_bundle("ИНТЕРФЕЙС".into(), UiAction::Tab(SettingsTab::Interface)));
                p.spawn(label_bundle("────────────────────────".into(), 13.0));
                match model.tab {
                    SettingsTab::Controls => {
                        p.spawn(label_bundle(format!("Чувствительность: {:.1}", settings.sensitivity), 16.0));
                        p.spawn(button_bundle("Чувствительность −".into(), UiAction::Adjust(SettingKey::Sensitivity, -0.1)));
                        p.spawn(button_bundle("Чувствительность +".into(), UiAction::Adjust(SettingKey::Sensitivity, 0.1)));
                        p.spawn(label_bundle(format!("Размер джойстика: {:.1}", settings.joystick_size), 16.0));
                        p.spawn(button_bundle("Размер джойстика − / +".into(), UiAction::Adjust(SettingKey::JoystickSize, 0.1)));
                        p.spawn(button_bundle(format!("Инверсия Y: {}", if settings.invert_y { "ВКЛ" } else { "ВЫКЛ" }), UiAction::Toggle(SettingKey::InvertY)));
                    }
                    SettingsTab::Vhs => {
                        p.spawn(button_bundle(format!("VHS: {}", if settings.vhs_enabled { "ВКЛ" } else { "ВЫКЛ" }), UiAction::Toggle(SettingKey::VhsEnabled)));
                        p.spawn(label_bundle(format!("Интенсивность: {:.2}", settings.vhs_intensity), 16.0));
                        p.spawn(button_bundle("Интенсивность −".into(), UiAction::Adjust(SettingKey::VhsIntensity, -0.05)));
                        p.spawn(button_bundle("Интенсивность +".into(), UiAction::Adjust(SettingKey::VhsIntensity, 0.05)));
                        p.spawn(label_bundle(format!("Scanlines: {:.2}  Noise: {:.2}", settings.scanlines, settings.noise), 15.0));
                        p.spawn(button_bundle("Scanlines +".into(), UiAction::Adjust(SettingKey::Scanlines, 0.05)));
                        p.spawn(button_bundle("Noise +".into(), UiAction::Adjust(SettingKey::Noise, 0.05)));
                        p.spawn(label_bundle(format!("Chromatic: {:.2}  Distortion: {:.2}", settings.chromatic, settings.distortion), 15.0));
                        p.spawn(button_bundle("Chromatic +".into(), UiAction::Adjust(SettingKey::Chromatic, 0.05)));
                        p.spawn(button_bundle("Distortion +".into(), UiAction::Adjust(SettingKey::Distortion, 0.05)));
                        p.spawn(button_bundle("Flicker +".into(), UiAction::Adjust(SettingKey::Flicker, 0.02)));
                    }
                    SettingsTab::Graphics => {
                        p.spawn(label_bundle(format!("Дальность: {:.0} м", settings.render_distance), 16.0));
                        p.spawn(button_bundle("Дальность −".into(), UiAction::Adjust(SettingKey::RenderDistance, -5.0)));
                        p.spawn(button_bundle("Дальность +".into(), UiAction::Adjust(SettingKey::RenderDistance, 5.0)));
                        p.spawn(button_bundle(format!("Тени: {}", if settings.shadows { "ВКЛ" } else { "ВЫКЛ" }), UiAction::Toggle(SettingKey::Shadows)));
                        p.spawn(label_bundle(format!("Лимит FPS: {}", settings.fps_limit), 16.0));
                        p.spawn(button_bundle("Переключить лимит FPS".into(), UiAction::Adjust(SettingKey::FpsLimit, 1.0)));
                    }
                    SettingsTab::Audio => {
                        p.spawn(label_bundle(format!("Общая громкость: {:.1}", settings.master_volume), 16.0));
                        p.spawn(button_bundle("Общая громкость −".into(), UiAction::Adjust(SettingKey::MasterVolume, -0.1)));
                        p.spawn(button_bundle("Общая громкость +".into(), UiAction::Adjust(SettingKey::MasterVolume, 0.1)));
                        p.spawn(label_bundle(format!("Окружение: {:.1}  Эффекты: {:.1}", settings.environment_volume, settings.effects_volume), 15.0));
                        p.spawn(button_bundle("Громкость окружения +".into(), UiAction::Adjust(SettingKey::EnvironmentVolume, 0.1)));
                        p.spawn(button_bundle("Громкость эффектов +".into(), UiAction::Adjust(SettingKey::EffectsVolume, 0.1)));
                        p.spawn(label_bundle(format!("Музыка: {:.1}", settings.music_volume), 15.0));
                        p.spawn(button_bundle("Музыка − / +".into(), UiAction::Adjust(SettingKey::MusicVolume, 0.1)));
                    }
                    SettingsTab::Interface => {
                        p.spawn(label_bundle(format!("Масштаб интерфейса: {:.1}", settings.ui_scale), 16.0));
                        p.spawn(button_bundle("Масштаб −".into(), UiAction::Adjust(SettingKey::UiScale, -0.1)));
                        p.spawn(button_bundle("Масштаб +".into(), UiAction::Adjust(SettingKey::UiScale, 0.1)));
                        p.spawn(button_bundle(format!("FPS на экране: {}", if settings.show_fps { "ВКЛ" } else { "ВЫКЛ" }), UiAction::Toggle(SettingKey::ShowFps)));
                    }
                }
                p.spawn(button_bundle("СБРОСИТЬ НАСТРОЙКИ".into(), UiAction::Reset));
                p.spawn(button_bundle("НАЗАД".into(), UiAction::Page(UiPage::Menu)));
            }
            UiPage::DebugWorld => {
                p.spawn(label_bundle("DEBUG WORLD".into(), 40.0));
                p.spawn(label_bundle("Отдельная тестовая сцена".into(), 17.0));
                p.spawn(label_bundle("TEXTURES  •  COLLISIONS  •  GLITCH PATCHES".into(), 13.0));
                p.spawn(label_bundle("Двигайся левым джойстиком, осматривайся свайпом справа.".into(), 14.0));
                p.spawn(button_bundle("ВЕРНУТЬСЯ В МЕНЮ".into(), UiAction::Page(UiPage::Menu)));
            }
            UiPage::Playing => {
                p.spawn(label_bundle("ИГРА".into(), 42.0));
                p.spawn(label_bundle("Сюжетная сцена пока не подключена.".into(), 18.0));
                p.spawn(label_bundle("Сейчас доступен отдельный тестовый мир.".into(), 15.0));
                p.spawn(button_bundle("ОТКРЫТЬ DEBUG WORLD".into(), UiAction::Page(UiPage::DebugWorld)));
                p.spawn(button_bundle("НАЗАД".into(), UiAction::Page(UiPage::Menu)));
            }
        }
    });
}

fn apply_ui_command(
    mut command: ResMut<UiCommand>,
    mut model: ResMut<UiModel>,
    mut settings: ResMut<Settings>,
) {
    let Some(action) = command.0.take() else { return; };
    match action {
        UiAction::Page(page) => model.page = page,
        UiAction::Tab(tab) => model.tab = tab,
        UiAction::Reset => *settings = Settings::default(),
        UiAction::Toggle(key) => match key {
            SettingKey::InvertY => settings.invert_y = !settings.invert_y,
            SettingKey::VhsEnabled => settings.vhs_enabled = !settings.vhs_enabled,
            SettingKey::Shadows => settings.shadows = !settings.shadows,
            SettingKey::ShowFps => settings.show_fps = !settings.show_fps,
            _ => {}
        },
        UiAction::Adjust(key, amount) => match key {
            SettingKey::Sensitivity => settings.sensitivity = (settings.sensitivity + amount).clamp(0.2, 3.0),
            SettingKey::JoystickSize => settings.joystick_size = (settings.joystick_size + amount).clamp(0.7, 1.6),
            SettingKey::VhsIntensity => settings.vhs_intensity = (settings.vhs_intensity + amount).clamp(0.0, 1.0),
            SettingKey::Scanlines => settings.scanlines = (settings.scanlines + amount).clamp(0.0, 1.0),
            SettingKey::Noise => settings.noise = (settings.noise + amount).clamp(0.0, 1.0),
            SettingKey::Chromatic => settings.chromatic = (settings.chromatic + amount).clamp(0.0, 1.0),
            SettingKey::Distortion => settings.distortion = (settings.distortion + amount).clamp(0.0, 1.0),
            SettingKey::Flicker => settings.flicker = (settings.flicker + amount).clamp(0.0, 1.0),
            SettingKey::RenderDistance => settings.render_distance = (settings.render_distance + amount).clamp(15.0, 100.0),
            SettingKey::FpsLimit => settings.fps_limit = match settings.fps_limit { 30 => 60, 60 => 90, 90 => 120, _ => 30 },
            SettingKey::MasterVolume => settings.master_volume = (settings.master_volume + amount).clamp(0.0, 1.0),
            SettingKey::EnvironmentVolume => settings.environment_volume = (settings.environment_volume + amount).clamp(0.0, 1.0),
            SettingKey::EffectsVolume => settings.effects_volume = (settings.effects_volume + amount).clamp(0.0, 1.0),
            SettingKey::MusicVolume => settings.music_volume = (settings.music_volume + amount).clamp(0.0, 1.0),
            SettingKey::UiScale => settings.ui_scale = (settings.ui_scale + amount).clamp(0.8, 1.4),
            SettingKey::JoystickOpacity => settings.joystick_opacity = (settings.joystick_opacity + amount).clamp(0.1, 1.0),
            SettingKey::InvertY | SettingKey::VhsEnabled | SettingKey::Shadows | SettingKey::ShowFps => {}
        },
    }
    settings.save();
    model.revision += 1;
}

fn rebuild_ui(
    mut commands: Commands,
    model: Res<UiModel>,
    settings: Res<Settings>,
    roots: Query<Entity, With<UiRoot>>,
    mut last_revision: Local<Option<u64>>,
) {
    if *last_revision == Some(model.revision) { return; }
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    spawn_ui(&mut commands, &model, &settings);
    *last_revision = Some(model.revision);
}

fn spawn_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
    size: Vec3,
    color: Color,
    collidable: bool,
) {
    let mut entity = commands.spawn((
        WorldEntity,
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(color)),
        Transform::from_translation(center),
    ));
    if collidable {
        entity.insert(Collider { center, half: size * 0.5 });
    }
}

fn sync_world(
    mut commands: Commands,
    model: Res<UiModel>,
    mut last_page: Local<Option<UiPage>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_entities: Query<Entity, With<WorldEntity>>,
) {
    if *last_page == Some(model.page) { return; }
    for entity in &world_entities { commands.entity(entity).despawn(); }
    spawn_scene_for_page(&mut commands, model.page, &mut meshes, &mut materials);
    *last_page = Some(model.page);
}

fn spawn_scene_for_page(
    commands: &mut Commands,
    page: UiPage,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let wall = Color::srgb(0.68, 0.61, 0.39);
    let carpet = Color::srgb(0.30, 0.25, 0.16);
    let ceiling = Color::srgb(0.70, 0.69, 0.58);
    spawn_box(commands, meshes, materials, Vec3::new(0.0, -0.12, -5.0), Vec3::new(12.0, 0.2, 24.0), carpet, false);
    spawn_box(commands, meshes, materials, Vec3::new(-6.0, 1.5, -5.0), Vec3::new(0.2, 3.0, 24.0), wall, true);
    spawn_box(commands, meshes, materials, Vec3::new(6.0, 1.5, -5.0), Vec3::new(0.2, 3.0, 24.0), wall, true);
    spawn_box(commands, meshes, materials, Vec3::new(0.0, 3.05, -5.0), Vec3::new(12.0, 0.2, 24.0), ceiling, false);
    for z in [-2.0, -7.0, -12.0, -17.0] {
        spawn_box(commands, meshes, materials, Vec3::new(0.0, 2.92, z), Vec3::new(1.5, 0.06, 0.7), Color::srgb(0.88, 0.84, 0.67), false);
    }

    if page == UiPage::DebugWorld {
        // Material samples and collision test objects.
        let samples = [
            (Vec3::new(-3.6, 0.8, -3.0), Color::srgb(0.73, 0.64, 0.40)),
            (Vec3::new(-2.2, 0.8, -3.0), Color::srgb(0.54, 0.48, 0.30)),
            (Vec3::new(-0.8, 0.8, -3.0), Color::srgb(0.39, 0.34, 0.24)),
            (Vec3::new(0.8, 0.8, -3.0), Color::srgb(0.18, 0.15, 0.12)),
        ];
        for (pos, color) in samples {
            spawn_box(commands, meshes, materials, pos, Vec3::splat(1.2), color, true);
        }
        // Door test: frame, panel and handle.
        spawn_box(commands, meshes, materials, Vec3::new(3.5, 1.35, -7.0), Vec3::new(1.8, 2.7, 0.18), wall, true);
        spawn_box(commands, meshes, materials, Vec3::new(3.5, 1.25, -6.88), Vec3::new(1.25, 2.5, 0.10), Color::srgb(0.42, 0.34, 0.22), true);
        spawn_box(commands, meshes, materials, Vec3::new(4.0, 1.2, -6.78), Vec3::new(0.08, 0.08, 0.08), Color::srgb(0.82, 0.72, 0.42), false);
        // Localized glitch patches, deliberately sparse and irregular.
        for (pos, size, color) in [
            (Vec3::new(-5.86, 1.9, -5.2), Vec3::new(0.035, 0.32, 0.72), Color::srgb(0.20, 0.24, 0.20)),
            (Vec3::new(-5.86, 1.45, -8.7), Vec3::new(0.035, 0.16, 0.42), Color::srgb(0.75, 0.69, 0.43)),
            (Vec3::new(5.86, 2.2, -11.3), Vec3::new(0.035, 0.42, 0.58), Color::srgb(0.14, 0.18, 0.16)),
        ] {
            spawn_box(commands, meshes, materials, pos, size, color, false);
        }
        // Low collision barriers.
        spawn_box(commands, meshes, materials, Vec3::new(-2.0, 0.35, -9.0), Vec3::new(1.2, 0.7, 0.7), Color::srgb(0.36, 0.31, 0.22), true);
        spawn_box(commands, meshes, materials, Vec3::new(0.0, 0.55, -10.5), Vec3::new(1.0, 1.1, 1.0), Color::srgb(0.45, 0.39, 0.26), true);
    }
}

fn mobile_controls(
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    colliders: Query<&Collider>,
    model: Res<UiModel>,
    settings: Res<Settings>,
    time: Res<Time>,
    mut touch_state: ResMut<TouchState>,
    mut look: ResMut<CameraLook>,
) {
    if !matches!(model.page, UiPage::DebugWorld | UiPage::Playing) { return; }
    let Ok(window) = windows.single() else { return; };
    let width = window.width();
    let mut look_delta = Vec2::ZERO;

    for touch in touches.iter() {
        let pos = touch.position();
        if touch.just_pressed() && pos.x < width * 0.48 && touch_state.left_id.is_none() {
            touch_state.left_id = Some(touch.id());
            touch_state.left_origin = pos;
        }
        if touch_state.left_id == Some(touch.id()) {
            if touch.just_released() {
                touch_state.left_id = None;
                touch_state.axis = Vec2::ZERO;
            } else {
                touch_state.axis = ((pos - touch_state.left_origin) / 90.0).clamp_length_max(1.0);
            }
        } else if pos.x >= width * 0.48 {
            look_delta += touch.delta();
        }
    }

    let sens = settings.sensitivity * 0.0025;
    look.yaw -= look_delta.x * sens;
    let y_sign = if settings.invert_y { -1.0 } else { 1.0 };
    look.pitch = (look.pitch - look_delta.y * sens * y_sign).clamp(-1.25, 1.25);

    let Ok(mut transform) = camera.single_mut() else { return; };
    transform.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);

    let axis = touch_state.axis;
    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();
    let movement = (right * axis.x - forward * axis.y).normalize_or_zero();
    let desired = transform.translation + movement * 3.2 * time.delta_secs();

    let player_half = Vec3::new(0.28, 0.75, 0.28);
    let blocked = colliders.iter().any(|c| {
        let a_min = desired - player_half;
        let a_max = desired + player_half;
        let b_min = c.center - c.half;
        let b_max = c.center + c.half;
        a_min.x < b_max.x && a_max.x > b_min.x &&
        a_min.y < b_max.y && a_max.y > b_min.y &&
        a_min.z < b_max.z && a_max.z > b_min.z
    });
    if !blocked { transform.translation = desired; }
}

fn update_vhs_tint(
    settings: Res<Settings>,
    mut roots: Query<&mut BackgroundColor, With<UiRoot>>,
) {
    if !settings.is_changed() { return; }
    for mut color in &mut roots {
        let alpha = if settings.vhs_enabled { 0.78 + settings.vhs_intensity * 0.08 } else { 0.78 };
        *color = BackgroundColor(Color::srgba(0.035, 0.035, 0.025, alpha));
    }
}
