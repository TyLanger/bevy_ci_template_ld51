use bevy::sprite::collide_aabb::collide;
use bevy::ui::FocusPolicy;
//use crate::GameState;
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use std::collections::HashMap;
// what is std::hashmap
// vs bevy utils hashmap?
// are they the same?

use crate::gold::GoldPile;
use crate::tower::Tower;
use crate::MouseWorldPos;
pub struct HexPlugin;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HexSpawnEvent>()
            .insert_resource(HexCollection {
                hexes: HashMap::new(),
            })
            .add_startup_system(spawn_hexes_circle)
            .add_system(spawn_ring_over_time)
            .add_system(spawn_hex)
            .add_system(hex_intro)
            .add_system(highlight_selection.before(select_hex))
            .add_system(select_hex)
            .add_system(gather_gold)
            .add_system(info_panel)
            .add_system(remove_old_panel)
            .add_system(test_from_pos);
        // .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_hexes_circle))
        // .add_system_set(SystemSet::on_update(GameState::Playing).with_system(spawn_hex));
    }
}

pub const DEG_TO_RAD: f32 = 0.01745;
const HEX_SPACING: f32 = 0.866_025_4;
const HEX_RADIUS: f32 = 27.0; // 20.0
const HEX_MARGIN: f32 = 0.0; // 0.4

pub struct HexSpawnEvent {
    coords: HexCoords,
}

#[derive(Component)]
pub struct Hex {
    radius: f32,
    pub coords: HexCoords,
    // gold available to be mined
    pub gold: u32,
    max_gold: u32,
    // when gold increments
    timer: Timer,
}

impl Hex {
    pub fn new(radius: f32, coords: HexCoords) -> Self {
        Hex {
            radius,
            coords,
            gold: 1,
            max_gold: 3,
            timer: Timer::from_seconds(7.5, true),
        }
    }

    pub fn mine(&mut self) -> bool {
        if self.gold > 1 {
            self.gold -= 1;
            return true;
        }
        false
    }
}

pub struct HexCollection {
    pub hexes: HashMap<HexCoords, Entity>,
}

fn gather_gold(mut q_hexes: Query<&mut Hex>, time: Res<Time>) {
    for mut hex in q_hexes.iter_mut() {
        if hex.timer.tick(time.delta()).just_finished() && hex.gold < hex.max_gold {
            hex.gold += 1;
        }
    }
}

fn spawn_hexes_circle(mut ev_spawn: EventWriter<HexSpawnEvent>) {
    let r = 4;
    let hex = HexCoords::new();

    for i in 0..r {
        let ring = hex.get_ring(i);
        for h in ring.iter() {
            ev_spawn.send(HexSpawnEvent { coords: *h });
        }
    }
}

// fn spawn_hexes_simple_diamond(mut ev_spawn: EventWriter<HexSpawnEvent>) {
//     for i in -3..=3 {
//         for j in -3..=3 {
//             let h = HexCoords { u: i, v: j };
//             println!("Hex coords: {:?}, position: {:?}", h, h.to_position());
//             ev_spawn.send(HexSpawnEvent { coords: h });
//         }
//     }
// }

struct TimedHexSpawner {
    timer: Timer,
    radius: u32,
    ring: Vec<HexCoords>,
}

impl Default for TimedHexSpawner {
    fn default() -> Self {
        TimedHexSpawner {
            timer: Timer::from_seconds(3.0, true),
            radius: 4,
            ring: Vec::new(),
        }
    }
}

fn spawn_ring_over_time(
    mut local: Local<TimedHexSpawner>,
    time: Res<Time>,
    mut ev_spawn: EventWriter<HexSpawnEvent>,
) {
    // don't grow forever
    if local.radius > 10 {
        return;
    }

    // it should build a whole side in 10s
    // a ring should take a minute
    // timer = 60 / len
    if local.ring.is_empty() {
        local.ring = HexCoords::new().get_ring(local.radius);
        local.ring.reverse();
        let len = local.ring.len();
        if len > 0 {
            local.timer = Timer::from_seconds(60.0 / len as f32, true);
        }
        local.radius += 1;
    }

    if local.timer.tick(time.delta()).just_finished() {
        let h = local.ring.pop();
        if let Some(h) = h {
            ev_spawn.send(HexSpawnEvent { coords: h });
        }
    }
}

fn spawn_hex(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_spawn: EventReader<HexSpawnEvent>,
    asset_server: Res<AssetServer>,
    mut hex_collect: ResMut<HexCollection>,
) {
    for (i, ev) in ev_spawn.iter().enumerate() {
        //println!("HexSpawnEvent");
        let entity = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(HEX_RADIUS - HEX_MARGIN, 6).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.6, 0.2, 0.7))),
                transform: Transform::from_translation(
                    ev.coords.to_position().extend(0.0) + Vec3::new(0.0, -100.0, 0.0),
                )
                .with_rotation(Quat::from_rotation_z(30.0 * DEG_TO_RAD)),
                ..default()
            })
            .insert(Hex::new(HEX_RADIUS, ev.coords))
            .insert(HexMover {
                start: ev.coords.to_position().extend(0.0) + Vec3::new(0.0, -100.0, 0.0),
                target: ev.coords.to_position().extend(0.0),
                timer: Timer::new(Duration::from_secs_f32(2.0 + 0.01 * i as f32), false),
            })
            .with_children(|parent| {
                parent.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("sprites/Hex_15_13.png"),
                    transform: Transform {
                        // spawn on top of the underlying hex
                        translation: Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.01,
                        },
                        // undo the hex's rotation
                        rotation: Quat::from_rotation_z(-30.0 * DEG_TO_RAD),
                        ..default()
                    },
                    ..default()
                });
            })
            .id();

        hex_collect.hexes.insert(ev.coords, entity);
    }
}

#[derive(Component)]
struct HexMover {
    start: Vec3,
    target: Vec3,
    timer: Timer,
}

fn hex_intro(
    mut commands: Commands,
    mut q_hexes: Query<(Entity, &mut Transform, &mut HexMover)>,
    time: Res<Time>,
) {
    for (e, mut t, mut h) in q_hexes.iter_mut() {
        if h.timer.tick(time.delta()).just_finished() {
            commands.entity(e).remove::<HexMover>();
        }
        // lerp to position
        //t.translation = h.start * (1.0 - h.timer.percent()) + h.target * h.timer.percent();
        // fancy
        // let p = h.timer.percent() * h.timer.percent();
        // t.translation = h.start * (1.0 - p) + h.target * p;
        // out and back
        let ease = ease_out_back(h.timer.percent());
        t.translation = h.start * (1.0 - ease) + h.target * ease;
    }
}

fn ease_out_back(t: f32) -> f32 {
    let a = 1.70158;
    let b = a + 1.0;

    1.0 + b * (t - 1.0).powi(3) + a * (t - 1.0).powi(2)
}

// right click to spawn at mouse pos?
// or spawn at selection?
// destroy when another hex becomes a selection?

fn info_panel(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    window: Res<Windows>,
    asset_server: Res<AssetServer>,
    q_selection: Query<(Option<&Tower>, Option<&GoldPile>), With<Selection>>,
) {
    if input.just_pressed(MouseButton::Right) {
        let win = window.get_primary().unwrap();
        if let Some(screen_pos) = win.cursor_position() {
            let font = asset_server.load("fonts/FiraSans-Bold.ttf");
            let mut text = "This is a Hex";
            for (tower, gold_pile) in q_selection.iter() {
                if tower.is_some() {
                    text = "This is a Tower";
                } else if gold_pile.is_some() {
                    text = "This is a Gold Pile";
                }
            }
            commands
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(screen_pos.x),
                            bottom: Val::Px(screen_pos.y),
                            ..default()
                        },
                        size: Size::new(Val::Auto, Val::Auto),
                        ..default()
                    },
                    color: Color::ORANGE.into(),
                    focus_policy: FocusPolicy::Block,
                    ..default()
                })
                .insert(InfoPanel)
                .insert(Interaction::Hovered)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font,
                                font_size: 15.0,
                                color: Color::WHITE,
                            },
                        ),
                        style: Style {
                            margin: UiRect::new(
                                Val::Px(3.0),
                                Val::Px(3.0),
                                Val::Px(2.0),
                                Val::Px(2.0),
                            ),
                            ..default()
                        },
                        // don't need this.
                        // the problem was using Added<Selection>
                        // I was never clicking on the same frame
                        //focus_policy: FocusPolicy::Pass,
                        ..default()
                    });
                });
        }
    }
}

#[derive(Component)]
struct InfoPanel;

fn remove_old_panel(
    mut commands: Commands,
    q_selection: Query<(), With<Selection>>,
    q_panels: Query<(Entity, &Interaction), With<InfoPanel>>,
) {
    // Add Interaction to the root node to have it track the mouse
    // like a button would
    // button is just a tag. Interaction does all the logic

    // why do I need this?
    // If I remove it, the panel deletes immediately.
    if !q_selection.is_empty() {
        for (ent, interaction) in q_panels.iter() {
            match *interaction {
                Interaction::Clicked => {
                    println!("Clicked panel");
                }
                Interaction::Hovered => {
                    // do nothing while hovered
                }
                Interaction::None => {
                    // delete it now
                    //println!("Delete the panel");
                    commands.entity(ent).despawn_recursive();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Selection;

fn highlight_selection(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_hex: Query<(&mut Handle<ColorMaterial>, Option<&Selection>), With<Hex>>,
) {
    for (color_handle, select) in q_hex.iter_mut() {
        if select.is_some() {
            let mut color_mat = materials.get_mut(&color_handle).unwrap();
            color_mat.color = Color::ANTIQUE_WHITE;
        } else {
            let mut color_mat = materials.get_mut(&color_handle).unwrap();
            color_mat.color = Color::rgb(0.388, 0.78, 0.3);
        }
    }
}

fn select_hex(
    mut commands: Commands,
    q_hex: Query<(Entity, &Transform, &Hex)>,
    q_selection: Query<(Entity, &Transform, &Hex), With<Selection>>,
    mouse: Res<MouseWorldPos>,
) {
    for (ent, trans, hex) in q_selection.iter() {
        if collide(
            mouse.0.extend(0.0),
            Vec2::new(0.1, 0.1),
            trans.translation,
            Vec2::new(1.6 * hex.radius, 1.8 * hex.radius),
        )
        .is_some()
        {
            return;
        } else {
            commands.entity(ent).remove::<Selection>();
        }
    }

    for (ent, trans, hex) in q_hex.iter() {
        // bounding box for hexes is close enough
        // 1.6 so you don't select multiple.
        if collide(
            mouse.0.extend(0.0),
            Vec2::new(0.1, 0.1),
            trans.translation,
            Vec2::new(1.6 * hex.radius, 1.6 * hex.radius),
        )
        .is_some()
        {
            commands.entity(ent).insert(Selection);
            return;
        }
    }
}

fn test_from_pos(input: Res<Input<KeyCode>>, mouse: Res<MouseWorldPos>) {
    if input.just_pressed(KeyCode::V) {
        println!(
            "Test at pos: {:?}. hex: {:?}",
            mouse.0,
            HexCoords::from_position(mouse.0)
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct HexCoords {
    // u is left-right offset
    // pos u is north-east
    // neg u is south-west
    // v is up and down
    u: isize,
    v: isize,
}

impl HexCoords {
    // this should be default
    // if the elements are isize, can I call default without making
    // a default impl?
    pub fn new() -> Self {
        HexCoords { u: 0, v: 0 }
    }

    pub fn to_position(self) -> Vec2 {
        Vec2::new(
            (HEX_RADIUS + HEX_RADIUS * 0.5) * (self.u as f32),
            2.0 * HEX_SPACING * HEX_RADIUS * (self.v as f32)
                + HEX_SPACING * HEX_RADIUS * (self.u as f32),
            // y = 2*u*h + v*h
            // h = radius * 0.86
        )
    }

    pub fn from_position(pos: Vec2) -> Self {
        // seems to work well enough
        // at least as consistent as selection

        let u = pos.x / (HEX_RADIUS + HEX_RADIUS * 0.5);
        let v = (pos.y - HEX_SPACING * HEX_RADIUS * u) / (2.0 * HEX_SPACING * HEX_RADIUS);
        //println!("(u, v) f32: ({:?}, {:?})", u, v);
        HexCoords {
            u: u.round() as isize,
            v: v.round() as isize,
        }
    }

    // pub fn equals(self, other: HexCoords) -> bool {
    //     return self.u == other.u && self.v == other.v;
    // }

    fn get_north(self, distance: isize) -> HexCoords {
        HexCoords {
            u: self.u,
            v: self.v + distance,
        }
    }

    fn get_north_east(self, distance: isize) -> HexCoords {
        HexCoords {
            u: self.u + distance,
            v: self.v,
        }
    }

    fn get_south_east(self, distance: isize) -> HexCoords {
        HexCoords {
            u: self.u + distance,
            v: self.v - distance,
        }
    }

    fn get_south(self, distance: isize) -> HexCoords {
        HexCoords {
            u: self.u,
            v: self.v - distance,
        }
    }

    fn get_south_west(self, distance: isize) -> HexCoords {
        HexCoords {
            u: self.u - distance,
            v: self.v,
        }
    }

    fn get_north_west(self, distance: isize) -> HexCoords {
        HexCoords {
            u: self.u - distance,
            v: self.v + distance,
        }
    }

    // radius 0 is self
    // radius 1 is neighbours
    /// Returns the ring around this hex
    ///
    /// # Arguments
    ///
    /// * `radius` - the radius of the ring around you.
    /// Returned in clockwise order starting with the north hex
    ///
    /// Radius of 0 is self.
    /// Radius of 1 is the 6 neighbours.
    /// Radius of 2 is the 12 hexes around the neighbours.
    pub fn get_ring(self, radius: u32) -> Vec<HexCoords> {
        let mut output = Vec::new();
        let radius = radius as isize;
        match radius {
            0 => {
                output.push(self);
            }
            _ => {
                let mut n = self.get_north(radius);
                output.push(n);
                // 1..radius
                // skip the first
                for _ in 1..radius {
                    n = n.get_south_east(1);
                    output.push(n);
                }

                let mut ne = self.get_north_east(radius);
                output.push(ne);
                for _ in 1..radius {
                    ne = ne.get_south(1);
                    output.push(ne);
                }

                let mut se = self.get_south_east(radius);
                output.push(se);
                for _ in 1..radius {
                    se = se.get_south_west(1);
                    output.push(se);
                }

                let mut s = self.get_south(radius);
                output.push(s);
                for _ in 1..radius {
                    s = s.get_north_west(1);
                    output.push(s);
                }

                let mut sw = self.get_south_west(radius);
                output.push(sw);
                for _ in 1..radius {
                    sw = sw.get_north(1);
                    output.push(sw);
                }

                let mut nw = self.get_north_west(radius);
                output.push(nw);
                for _ in 1..radius {
                    nw = nw.get_north_east(1);
                    output.push(nw);
                }
            }
        }

        output
    }

    #[allow(dead_code)]
    pub fn get_neighbours(self) -> Vec<HexCoords> {
        self.get_ring(1)
    }
}
