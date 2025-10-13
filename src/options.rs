use bevy::prelude::*;
use crate::GameState;

// Plugin initializer for options systems
pub fn init_options_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::Options), setup_options)
        .add_systems(OnExit(GameState::Options), cleanup_options)
        .add_systems(
            Update,
            (options_button_system, options_button_interaction)
                .run_if(in_state(GameState::Options)),
        );
}

// Marker component for options entities
#[derive(Component)]
pub struct OptionsEntity;

// Component for options buttons
#[derive(Component)]
pub enum OptionsButton {
    Back,
}

// Setup options UI
pub fn setup_options(mut commands: Commands) {
    // Root node for the options menu
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
            OptionsEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("OPTIONS"),
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

            // Placeholder text for future options
            parent.spawn((
                Text::new("Settings coming soon..."),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Back button
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
                    OptionsButton::Back,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("BACK"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
        });
}

// Cleanup options entities
pub fn cleanup_options(mut commands: Commands, options_entities: Query<Entity, With<OptionsEntity>>) {
    for entity in options_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Handle options button interactions (hover effects)
pub fn options_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<OptionsButton>),
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

// Handle options button clicks
pub fn options_button_system(
    interaction_query: Query<(&Interaction, &OptionsButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                OptionsButton::Back => {
                    next_state.set(GameState::Menu);
                }
            }
        }
    }
}
