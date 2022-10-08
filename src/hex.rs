use bevy::sprite::collide_aabb::collide;
//use crate::GameState;
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::MouseWorldPos;
pub struct HexPlugin;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HexSpawnEvent>()
            .add_startup_system(spawn_hexes_circle)
            .add_system(spawn_hex)
            .add_system(hex_intro)
            .add_system(highlight_selection.before(select_hex))
            .add_system(select_hex)
            .add_system(gather_gold);
        // .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_hexes_circle))
        // .add_system_set(SystemSet::on_update(GameState::Playing).with_system(spawn_hex));
    }
}

pub const DEG_TO_RAD: f32 = 0.01745;
const HEX_SPACING: f32 = 0.866_025_4;
const HEX_RADIUS: f32 = 20.0;
const HEX_MARGIN: f32 = 0.4;

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

    // pub fn mine(&mut self) -> bool {
    //     if self.gold > 1 {
    //         self.gold -= 1;
    //         return true;
    //     }
    //     return false;
    // }
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

fn spawn_hex(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_spawn: EventReader<HexSpawnEvent>,
) {
    for (i, ev) in ev_spawn.iter().enumerate() {
        //println!("HexSpawnEvent");
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(HEX_RADIUS - HEX_MARGIN, 6).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.6, 0.2, 0.7))),
                transform: Transform::from_translation(ev.coords.to_position().extend(0.0))
                    .with_rotation(Quat::from_rotation_z(30.0 * DEG_TO_RAD)),
                ..default()
            })
            .insert(Hex::new(HEX_RADIUS, ev.coords))
            .insert(HexMover {
                start: ev.coords.to_position().extend(0.0) + Vec3::new(0.0, -100.0, 0.0),
                target: ev.coords.to_position().extend(0.0),
                timer: Timer::new(Duration::from_secs_f32(3.0 + 0.01 * i as f32), false),
            });
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
            color_mat.color = Color::GREEN;
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
        ).is_some() {
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
        ).is_some() {
            commands.entity(ent).insert(Selection);
            return;
        }
    }

}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
