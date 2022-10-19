use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::sprite::Anchor;
use bevy::utils::Duration;

use crate::boids::Boid;
use crate::enemies::{Boss, BossCapEvent, Dead, Enemy};
use crate::hex::{Hex, HexCollection, HexCoords, Selection, DEG_TO_RAD};
use crate::tower::{Tower, TowerPreview};
use crate::MouseWorldPos;
use crate::{palette::*, tower};

const GOLD_SPAWN_TIME: f32 = 10.0;

pub struct GoldPlugin;

impl Plugin for GoldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ModifySpawnerEvent>()
            .add_event::<PileCapEvent>()
            .add_event::<PileSpawnEvent>()
            .add_event::<PileRemoveEvent>()
            .add_event::<SpawnGoldEvent>()
            .add_event::<DelayedGoldEvent>()
            .add_event::<DelayedGoldEventHelper>()
            .add_startup_system(setup)
            .add_system(pile_input)
            .add_system(spawn_pile)
            .add_system(remove_pile)
            .add_system(generate_gold)
            .add_system(spawn_gold)
            .add_system(delay_gold)
            .add_system(delay_gold_helper)
            //.add_system(place_spawner)
            //.add_system(remove_spawner)
            //.add_system(check_spawner)
            .add_system(move_gold)
            .add_system(check_mouse)
            //.add_system(store_gold.before(enemies::grab_gold))
            // enemy.bullet_hit might break this. It was before enemy::grab
            // so I'm putting it before this
            .add_system(gold_collisions.before(tower::bullet_hit))
            // spawn a health bar before it gets destroyed
            // if you are holding gold when you spawna tower, it gets built instantly
            // and the health bar is removed before it is created
            .add_system(make_health_bar.before(gold_collisions))
            .add_system(animate_health_bar);
    }
}

#[derive(Component)]
pub struct GoldSpawner {
    timer: Timer,
    pub radius: u32,
    //gold_gen: u32,
}

impl GoldSpawner {
    pub fn new() -> Self {
        GoldSpawner {
            timer: Timer::new(Duration::from_secs_f32(GOLD_SPAWN_TIME), true),
            radius: 1,
            //gold_gen: 1,
        }
    }
}

pub struct SpawnGoldEvent {
    pub position: Vec3,
}

pub struct ModifySpawnerEvent {
    pub coords: HexCoords,
    //pub modification: Modification,
}

// pub enum Modification {
//     Remove,
//     Hide,
//     Upgrade,
// }

#[derive(Component)]
pub struct Gold;

#[derive(Component)]
pub struct MouseFollow;

// need to move mouse close to pick up gold
// but then need to move farther away to break the tether and drop it
const TETHER_BREAK_DIST: f32 = 250.0;
const TETHER_ENTER_DIST: f32 = 90.0;
const GOLD_MOVE_SPEED: f32 = 225.0;

#[derive(Component)]
pub struct GoldPile {
    pub count: u32,
    pub gold_cap: u32,
}

impl GoldPile {
    pub fn new(cap: u32) -> Self {
        GoldPile {
            count: 0,
            gold_cap: cap,
        }
    }
}

pub struct PileSpawnEvent {
    pub coords: HexCoords,
    starting_gold: u32,
}

impl PileSpawnEvent {
    pub fn new(coords: HexCoords) -> Self {
        PileSpawnEvent {
            coords,
            starting_gold: 0,
        }
    }
}

pub struct PileCapEvent {
    pub coords: HexCoords,
    pub amount: u32,
}

pub struct PileRemoveEvent {
    pub coords: HexCoords,
}

fn setup(mut ev_spawn: EventWriter<PileSpawnEvent>) {
    // spawn a pile at the center with some starting cash
    ev_spawn.send(PileSpawnEvent {
        coords: HexCoords::new(),
        starting_gold: 11,
    });
}

fn gold_collisions(
    mut commands: Commands,
    mut q_gold: Query<(Entity, &mut Transform), (Without<Enemy>, With<Gold>)>,
    mut q_pile: Query<
        (
            &Transform,
            &mut GoldPile,
            Option<&Hex>,
            Option<&TowerPreview>,
        ),
        Without<Gold>,
    >,
    mut q_enemies: Query<
        (Entity, &Transform, &mut Enemy),
        (Without<Gold>, Without<Dead>, Without<Boss>),
    >,
    mut ev_cap: EventWriter<PileCapEvent>,
    mut ev_boss_cap: EventWriter<BossCapEvent>,
) {
    for (gold_ent, mut gold_trans) in q_gold.iter_mut() {
        let mut gold_alive = true;
        for (pile_trans, mut pile, hex, preview) in q_pile.iter_mut() {
            let b_size: Vec2 = if hex.is_none() {
                // boss
                Vec2::new(80., 80.)
            } else {
                // normal pile
                Vec2::new(20., 20.)
            };

            if collide(
                gold_trans.translation,
                Vec2::new(8., 12.),
                pile_trans.translation,
                b_size,
            )
            .is_some()
                && pile.count < pile.gold_cap
            {
                pile.count += 1;
                //println!("Plink! {:?} e: {:?}", pile.count, gold_ent);
                commands.entity(gold_ent).despawn_recursive();
                if pile.count == pile.gold_cap {
                    //println!("Cap reached! {:?}", pile.count);
                    if let Some(hex) = hex {
                        ev_cap.send(PileCapEvent {
                            coords: hex.coords,
                            amount: pile.count,
                        });
                        if preview.is_some() {
                            pile.count = 0; // empty the pile to pay for tower
                            pile.gold_cap = 0; // don't accept any more
                        }
                    } else {
                        ev_boss_cap.send(BossCapEvent);
                    }
                }
                gold_alive = false;
                break;
            }
        }
        // has the gold already been deleted?
        if gold_alive {
            for (e_ent, e_trans, mut enemy) in q_enemies.iter_mut() {
                // don't do it this way again
                // despawn the gold
                // add a sprite
                // die and spawn a gold on the corpse

                // when you grab the gold, run away
                // directly away from 0,0 ?
                // remove the gold?
                // add something to the enemy so they don't pick up more gold?
                if enemy.has_gold {
                    // can't pick up multiple gold
                    break;
                }
                if collide(
                    gold_trans.translation,
                    Vec2::new(8., 12.),
                    e_trans.translation,
                    Vec2::new(15., 15.),
                )
                .is_some()
                {
                    //println!("Grabbed a gold: ent: {:?}", gold_ent);
                    enemy.has_gold = true;

                    commands.entity(gold_ent).remove::<Gold>();

                    commands.entity(e_ent).add_child(gold_ent);
                    gold_trans.translation = Vec3::new(0.0, 0.0, 0.1);
                    break;
                }
            }
        }
    }
}

#[derive(Component)]
struct PileSprite;

fn spawn_pile(
    mut commands: Commands,
    mut ev_spawn: EventReader<PileSpawnEvent>,
    q_hexes: Query<
        Entity,
        (
            With<Hex>,
            (Without<TowerPreview>, Without<Tower>, Without<GoldPile>),
        ),
    >,
    hex_collect: Res<HexCollection>,
) {
    // don't run before hexes exist
    // this preserves the event that is send frame ~1
    // until hexes exist on frame ~2
    // then on frame ~3 this runs
    // or maybe frame ~2 if this system happens to run after the hex spawn system
    if !q_hexes.is_empty() {
        for ev in ev_spawn.iter() {
            if let Some(&e) = hex_collect.hexes.get(&ev.coords) {
                if let Ok(ent) = q_hexes.get(e) {
                    // for (ent, hex) in q_hexes.iter() {
                    //     if ev.coords == (hex.coords) {
                    commands
                        .entity(ent)
                        .insert(GoldPile {
                            count: ev.starting_gold,
                            gold_cap: 500,
                        })
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(SpriteBundle {
                                    sprite: Sprite {
                                        color: ORANGE,
                                        custom_size: Some(Vec2::new(20.0, 20.0)),
                                        ..default()
                                    },
                                    transform: Transform {
                                        // spawn on top of the underlying hex
                                        translation: Vec3 {
                                            x: 0.0,
                                            y: 0.0,
                                            z: 0.2,
                                        },
                                        // undo the hex's rotation
                                        rotation: Quat::from_rotation_z(-30.0 * DEG_TO_RAD),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .insert(PileSprite);
                        });
                }
            }
        }
    }
}

fn remove_pile(
    mut commands: Commands,
    mut ev_remove: EventReader<PileRemoveEvent>,
    mut ev_spawn_gold: EventWriter<SpawnGoldEvent>,
    q_piles: Query<(Entity, &Children, &Transform, &GoldPile), With<Hex>>,
    q_child: Query<(Entity, &Parent, Option<&HealthBar>, Option<&PileSprite>)>,
    hex_collect: Res<HexCollection>,
) {
    for ev in ev_remove.iter() {
        if let Some(&e) = hex_collect.hexes.get(&ev.coords) {
            if let Ok((ent, children, trans, pile)) = q_piles.get(e) {
                // for (ent, children, trans, hex, pile) in q_piles.iter() {
                //     if ev.coords == (hex.coords) {
                for _ in 0..pile.count {
                    ev_spawn_gold.send(SpawnGoldEvent {
                        position: trans.translation,
                    });
                }

                for &child in children {
                    if let Ok((child_ent, _parent, child_hp, child_pile_sprite)) =
                        q_child.get(child)
                    {
                        if child_hp.is_some() || child_pile_sprite.is_some() {
                            // delete if
                            // health bar
                            // or pile sprite
                            commands.entity(child_ent).despawn_recursive();
                        }
                    }
                }

                commands.entity(ent).remove::<GoldPile>();
            }
        }
    }
}

#[derive(Component)]
struct HealthBar {
    is_background: bool,
}

fn make_health_bar(mut commands: Commands, q_new: Query<(Entity, Option<&Boss>), Added<GoldPile>>) {
    for (ent, boss) in q_new.iter() {
        let mut r = Quat::from_rotation_z(-30.0 * DEG_TO_RAD);
        let mut y = 0.0;
        let mut x = -8.0;
        if boss.is_some() {
            r = Quat::default();
            y = -42.0;
            x = 0.0;
        }
        commands.entity(ent).with_children(|hex| {
            hex.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: GOLD,
                    custom_size: Some(Vec2::new(0.0, 6.0)), // 25.0
                    anchor: Anchor::Center,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x,
                        y: -12.0 + y,
                        z: 0.5,
                    },
                    rotation: r,
                    ..default()
                },
                ..default()
            })
            .insert(HealthBar {
                is_background: false,
            });

            hex.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: DARK_BLUE,
                    custom_size: Some(Vec2::new(27.0, 8.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x,
                        y: -12.0 + y,
                        z: 0.4,
                    },
                    rotation: r,
                    ..default()
                },
                ..default()
            })
            .insert(HealthBar {
                is_background: true,
            });
        });
    }
}

fn animate_health_bar(
    mut commands: Commands,
    mut q_bar: Query<(Entity, &HealthBar, &Parent, &mut Sprite)>,
    q_piles: Query<&GoldPile>,
) {
    for (ent, bar, parent, mut sprite) in q_bar.iter_mut() {
        if !bar.is_background {
            let pile = q_piles.get(parent.get());
            match pile {
                Ok(p) => {
                    let x = p.count as f32 / p.gold_cap as f32;
                    sprite.custom_size = Some(Vec2::new(x * 25.0, 6.0));
                }
                Err(_) => {
                    //println!("Error. No pile");
                    commands.entity(ent).despawn_recursive();
                }
            }
        }
    }
}

fn pile_input(
    input: Res<Input<KeyCode>>,
    mut ev_spawn: EventWriter<PileSpawnEvent>,
    mut ev_remove: EventWriter<PileRemoveEvent>,
    q_selection: Query<&Hex, With<Selection>>,
) {
    for hex in q_selection.iter() {
        if input.just_pressed(KeyCode::X) {
            ev_remove.send(PileRemoveEvent { coords: hex.coords });
        }
        if input.just_pressed(KeyCode::G) {
            ev_spawn.send(PileSpawnEvent::new(hex.coords));
        }
    }
}

fn generate_gold(
    mut q_gold_spawners: Query<(&Hex, &mut GoldSpawner)>,
    mut q_empty_hexes: Query<
        (&Transform, &mut Hex),
        (Without<Tower>, Without<GoldPile>, Without<GoldSpawner>),
    >,
    mut ev_gold_spawn: EventWriter<SpawnGoldEvent>,
    time: Res<Time>,
    hex_collect: Res<HexCollection>,
) {
    for (hex, mut spawner) in q_gold_spawners.iter_mut() {
        if spawner.timer.tick(time.delta()).just_finished() {
            // spawn around you
            let mut neighbours = Vec::new();
            for i in 1..=spawner.radius {
                neighbours.append(&mut hex.coords.get_ring(i));
            }
            // let mut neighbours = hex.coords.get_ring(1);
            // let mut outer_ring = hex.coords.get_ring(2);
            // neighbours.append(&mut outer_ring);
            // always 18
            // check if a hex really exists below
            //println!("neighbours len. expect 18: {:?}", neighbours.len());
            for &n in neighbours.iter() {
                // can probably replace with
                // hashmap
                // q_empty_hexes.get_many(neighbours)

                // check if I can spawn
                // does a hex at this coordinate exist?
                if let Some(&e) = hex_collect.hexes.get(&n) {
                    // if it does exist, does it match this query?
                    if let Ok((trans2, mut hex2)) = q_empty_hexes.get_mut(e) {
                        // empty space

                        // mine and return success
                        if hex2.mine() {
                            ev_gold_spawn.send(SpawnGoldEvent {
                                position: trans2.translation,
                                //frame: (i*10)+1,
                            });
                        }
                    }
                }
            }
        }
    }
}

struct DelayedGoldEvent {
    position: Vec3,
    frame: usize,
}

struct DelayedGoldEventHelper {
    position: Vec3,
    frame: usize,
}

fn delay_gold(
    mut ev_gold_in: EventReader<DelayedGoldEvent>,
    mut ev_gold_out: EventWriter<DelayedGoldEventHelper>,
    mut ev_gold_spawn: EventWriter<SpawnGoldEvent>,
) {
    // if !ev_gold_in.is_empty() {
    //     println!("delay gold, len: {:?}", ev_gold_in.len());
    // }
    for read in ev_gold_in.iter() {
        let frame = read.frame - 1;
        if frame == 0 {
            ev_gold_spawn.send(SpawnGoldEvent {
                position: read.position,
            });
        } else {
            ev_gold_out.send(DelayedGoldEventHelper {
                position: read.position,
                frame,
            });
        }
    }
}

// can't have Reader<A> and Writer<A> in the same system
// this is the loophole
// but it still doesn't look great
fn delay_gold_helper(
    mut ev_gold_in: EventReader<DelayedGoldEventHelper>,
    mut ev_gold_out: EventWriter<DelayedGoldEvent>,
    mut ev_gold_spawn: EventWriter<SpawnGoldEvent>,
) {
    // if !ev_gold_in.is_empty() {
    //     println!("delay gold helper, len: {:?}", ev_gold_in.len());
    // }
    for read in ev_gold_in.iter() {
        let frame = read.frame - 1;
        if frame == 0 {
            ev_gold_spawn.send(SpawnGoldEvent {
                position: read.position,
            });
        } else {
            ev_gold_out.send(DelayedGoldEvent {
                position: read.position,
                frame,
            });
        }
    }
}

fn spawn_gold(
    mut commands: Commands,
    mut ev_gold_spawn: EventReader<SpawnGoldEvent>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_gold_spawn.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("sprites/Gold2.png"),
                // sprite: Sprite {
                //     color: GOLD,
                //     custom_size: Some(Vec2::new(8.0, 12.)),
                //     ..default()
                // },
                transform: Transform {
                    translation: Vec3 {
                        x: ev.position.x,
                        y: ev.position.y,
                        z: 0.3,
                    },
                    scale: Vec3::ONE * 2.0,
                    ..default()
                },
                ..default()
            })
            .insert(Gold)
            .insert(Boid::new());
    }
}

fn check_mouse(
    mut commands: Commands,
    q_gold: Query<(Entity, &Transform, Option<&MouseFollow>), With<Gold>>,
    mouse: Res<MouseWorldPos>,
) {
    for (gold_ent, gold_trans, gold_follow) in q_gold.iter() {
        match gold_follow {
            Some(_) => {
                // following the mouse
                if Vec2::distance(gold_trans.translation.truncate(), mouse.0) > TETHER_BREAK_DIST {
                    commands.get_or_spawn(gold_ent).remove::<MouseFollow>();
                }
            }
            _ => {
                // not following

                if Vec2::distance(gold_trans.translation.truncate(), mouse.0) < TETHER_ENTER_DIST {
                    commands.get_or_spawn(gold_ent).insert(MouseFollow);
                }
            }
        }
    }
}

fn move_gold(
    mut q_gold: Query<&mut Transform, (With<Gold>, With<MouseFollow>)>,
    mouse: Res<MouseWorldPos>,
    time: Res<Time>,
) {
    for mut gold in q_gold.iter_mut() {
        let dir = mouse.0 - gold.translation.truncate();
        gold.translation +=
            dir.normalize_or_zero().extend(0.0) * GOLD_MOVE_SPEED * time.delta_seconds();
    }
}
