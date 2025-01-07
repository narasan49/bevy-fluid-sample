use bevy::prelude::*;

pub struct GameUiPlugin;

#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct AddButton;

const BUTTON_COLOR: Color = Color::LinearRgba(LinearRgba::WHITE);
const HOVERED_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 0.15,
    green: 0.15,
    blue: 0.15,
    alpha: 1.0,
});
const PRESSED_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 0.15,
    green: 0.15,
    blue: 0.75,
    alpha: 1.0,
});

fn basic_button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(100.0),
            height: Val::Px(30.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: BUTTON_COLOR.into(),
        ..default()
    }
}

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, button_update);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(basic_button())
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Reset",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ));
                })
                .insert(ResetButton);
            parent
                .spawn(basic_button())
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Add",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ));
                })
                .insert(AddButton);
        });
}

fn button_update(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_COLOR.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_COLOR.into();
            }
            Interaction::None => {
                *color = BUTTON_COLOR.into();
            }
        }
    }
}
