use bevy::prelude::*;

use crate::{enemies::BossCapEvent, StartSpawningEnemiesEvent};

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(start_menu)
            .add_startup_system(tutorial_side_bar)
            .add_event::<RemoveMenuEvent>()
            .insert_resource(AcceptInput(false))
            .add_system(button_system)
            .add_system(remove_start_menu)
            .add_system(allow_input)
            .add_system(win_menu)
            .add_system(toggle_tutorial);
    }
}

#[derive(Component)]
struct StartMenu;

#[derive(Component)]
struct RemoveButton;

#[derive(Component)]
struct ButtonInfo {
    base_text: String,
    hovered_text: String,
}

#[derive(Component)]
struct EndMenu;

struct RemoveMenuEvent;

pub struct AcceptInput(pub bool);

const NORMAL_BUTTOM: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTOM: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTOM: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_system(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &Children, Option<&RemoveButton>),
        (Changed<Interaction>, With<Button>),
    >,
    q_child: Query<&ButtonInfo>,
    mut q_text: Query<&mut Text>,
    mut ev_start: EventWriter<StartSpawningEnemiesEvent>,
    mut ev_remove: EventWriter<RemoveMenuEvent>,
) {
    for (interaction, mut color, children, start) in &mut q_interaction {
        let mut text = q_text.get_mut(children[0]).unwrap();
        let info = q_child.get(children[0]);
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTOM.into();
                println!("Button pressed");
                if start.is_some() {
                    ev_start.send(StartSpawningEnemiesEvent);
                    // don't let the player click through the menu
                }
                ev_remove.send(RemoveMenuEvent);
            }
            Interaction::Hovered => {
                text.sections[0].value = info.unwrap().hovered_text.clone();
                *color = HOVERED_BUTTOM.into();
            }
            Interaction::None => {
                text.sections[0].value = info.unwrap().base_text.clone();
                *color = NORMAL_BUTTOM.into();
            }
        }
    }
}

fn allow_input(
    mut ev_start: EventReader<StartSpawningEnemiesEvent>,
    mut accept: ResMut<AcceptInput>,
) {
    for _ev in ev_start.iter() {
        accept.0 = true;
    }
}

fn start_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::rgb(0.6, 0.6, 0.7).into(),
            ..default()
        })
        .insert(StartMenu)
        .with_children(|root| {
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    //align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,

                    ..default()
                },
                color: Color::NONE.into(), //:BEIGE.into(),
                ..default()
            })
            .with_children(|center| {
                center
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            // default is 1280x720
                            // 150/1280 = 11.7%
                            // 65/720 = 9%
                            // 12% of 40% = 30%
                            //size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                            size: Size::new(Val::Percent(12.0), Val::Percent(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        color: NORMAL_BUTTOM.into(),
                        ..default()
                    })
                    .insert(RemoveButton)
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle::from_section(
                                "Button",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ))
                            .insert(ButtonInfo {
                                base_text: "Start".to_string(),
                                hovered_text: "Game".to_string(),
                            });
                    });
                center.spawn_bundle(NodeBundle {
                    style: Style {
                        //size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                        size: Size::new(Val::Px(500.0), Val::Px(76.0)),
                        margin: UiRect::new(
                            Val::Auto,
                            Val::Auto,
                            Val::Percent(10.0),
                            Val::Percent(10.0),
                        ),
                        ..default()
                    },
                    image: UiImage(asset_server.load("sprites/Title.png")),
                    ..default()
                });
            });
        });
}

fn remove_start_menu(
    mut commands: Commands,
    mut ev_remove: EventReader<RemoveMenuEvent>,
    q_menu: Query<Entity, Or<(With<StartMenu>, With<EndMenu>)>>,
) {
    for _ev in ev_remove.iter() {
        for ent in q_menu.iter() {
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn win_menu(
    mut commands: Commands,
    mut ev_boss: EventReader<BossCapEvent>,
    asset_server: Res<AssetServer>,
) {
    for _ev in ev_boss.iter() {
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::NONE.into(),
                ..default()
            })
            .insert(EndMenu)
            .with_children(|root| {
                root.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
                        //align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,

                        ..default()
                    },
                    color: Color::NONE.into(), //:BEIGE.into(),
                    ..default()
                })
                .with_children(|center| {
                    center
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                // default is 1280x720
                                // 150/1280 = 11.7%
                                // 65/720 = 9%
                                // 12% of 40% = 30%
                                //size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                size: Size::new(Val::Percent(10.0), Val::Percent(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            color: NORMAL_BUTTOM.into(),
                            ..default()
                        })
                        //.insert(RemoveButton)
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(TextBundle::from_section(
                                    "Button",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 40.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                    },
                                ))
                                .insert(ButtonInfo {
                                    base_text: "You Win".to_string(),
                                    hovered_text: "Continue".to_string(),
                                });
                        });
                    center.spawn_bundle(NodeBundle {
                        style: Style {
                            //size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                            size: Size::new(Val::Px(500.0), Val::Px(76.0)),

                            margin: UiRect::new(
                                Val::Auto,
                                Val::Auto,
                                Val::Percent(10.0),
                                Val::Percent(10.0),
                            ),
                            ..default()
                        },
                        image: UiImage(asset_server.load("sprites/Title.png")),
                        ..default()
                    });
                });
            });
    }
}

fn tutorial_side_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "Tutorial:\n",
                TextStyle {
                    font: font.clone(),
                    font_size: 35.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\nLeft Click to place a tower preview.",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\nPress X on the orange square to destroy it.",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\nThe gold follows your mouse. Drag it over to the tower to pay for it and build it.",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\n\nA Tower mines the tiles around it for gold every 10s. Tiles only produce gold so fast so multiple towers mining the same tile has diminishing returns.",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\nPress G to spawn a Gold Pile to store gold for later.",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\nPress X to destroy something. You get back the gold in it. Fully built Towers only refund 80%.",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "\n\nPress Tab to toggle this menu",
                TextStyle {
                    font,
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
        ])
        .with_text_alignment(TextAlignment::CENTER_LEFT)
        .with_style(Style {
            //align_self: AlignSelf::Center,
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            max_size: Size {
                width: Val::Px(350.),
                height: Val::Undefined,
            },
            ..default()
        }),
    )
    .insert(Tutorial);
}

#[derive(Component)]
struct Tutorial;

fn toggle_tutorial(
    input: Res<Input<KeyCode>>,
    mut q_tutorial: Query<&mut Visibility, With<Tutorial>>,
) {
    if input.just_pressed(KeyCode::Tab) {
        for mut vis in q_tutorial.iter_mut() {
            vis.is_visible = !vis.is_visible;
        }
    }
}
