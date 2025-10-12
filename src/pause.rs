use bevy::prelude::*;
use crate::GameState;
use crate::gameplay::GameEntity;

// Marker component for pause menu entities
#[derive(Component)]
pub struct PauseEntity;

// Component for pause menu buttons
#[derive(Component)]
pub enum PauseButton {
    Resume,
    MainMenu,
}

// Handle pause input (ESC key)
pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

// Setup pause menu UI
pub fn setup_pause_menu(mut commands: Commands) {
    // Root node for the pause menu
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            // Resume button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    PauseButton::Resume,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("RESUME"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });

            // Main Menu button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    PauseButton::MainMenu,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("MAIN MENU"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
        });
}

// Cleanup pause menu entities
pub fn cleanup_pause_menu(mut commands: Commands, pause_entities: Query<Entity, With<PauseEntity>>) {
    for entity in pause_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Cleanup game entities when returning to menu from pause
pub fn cleanup_game_on_menu_return(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    next_state: Option<Res<NextState<GameState>>>,
) {
    // Only cleanup if we're transitioning to Menu
    if let Some(next) = next_state {
        if matches!(next.as_ref(), NextState::Pending(GameState::Menu)) {
            for entity in game_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

// Handle pause button interactions (hover effects)
pub fn pause_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<PauseButton>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::from(Color::srgb(0.6, 0.6, 0.7));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::from(Color::srgb(0.7, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::from(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

// Handle pause button clicks
pub fn pause_button_system(
    interaction_query: Query<(&Interaction, &PauseButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                PauseButton::Resume => {
                    next_state.set(GameState::Playing);
                }
                PauseButton::MainMenu => {
                    next_state.set(GameState::Menu);
                }
            }
        }
    }
}
