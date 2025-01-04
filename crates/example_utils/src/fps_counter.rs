use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[derive(Component)]
pub struct FpsRoot;

#[derive(Component)]
pub struct FpsText;

/// FPS counter plugin comming from Unofficial Bevy Cheat Book,
/// https://bevy-cheatbook.github.io/cookbook/print-framerate.html
pub struct FpsCounterPlugin;

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup_fps_text)
            .add_systems(Update, update_fps_text);
    }
}

fn setup_fps_text(mut commands: Commands) {
    let root = commands
        .spawn((
            FpsRoot,
            NodeBundle {
                background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(1.0),
                    top: Val::Percent(1.0),
                    bottom: Val::Auto,
                    left: Val::Auto,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let text = commands
        .spawn((
            FpsText,
            TextBundle {
                text: Text::from_sections([
                    TextSection {
                        value: "FPS: ".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    },
                    TextSection {
                        value: "N/A".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    },
                ]),
                ..default()
            },
        ))
        .id();
    commands.entity(root).push_children(&[text]);
}

fn update_fps_text(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[1].value = format!("{fps:>4.0}");
        } else {
            text.sections[1].value = "N/A".into();
        }
    }
}
