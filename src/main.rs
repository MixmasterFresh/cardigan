use bevy::prelude::*;

mod startup;
mod menu;
mod options;
mod pause;
mod gameplay;

use startup::*;
use menu::*;
use options::*;
use pause::*;
use gameplay::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Card Renderer".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(CardConfig {
            hover_scale: 1.3,
            animation_speed: 5.0,
        })
        .init_state::<GameState>();

    // Initialize systems from each module
    init_startup_systems(&mut app);
    init_menu_systems(&mut app);
    init_options_systems(&mut app);
    init_pause_systems(&mut app);
    init_gameplay_systems(&mut app);

    app.run();
}

// Game states
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    Options,
}
