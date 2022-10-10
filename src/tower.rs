use bevy::{prelude::*, sprite::collide_aabb::collide, utils::FloatOrd};

use crate::{
    enemies::{BossSpawnEvent, Dead, Enemy},
    gold::*,
    hex::*,
    tutorial::AcceptInput,
};

const TOWER_COST_GROWTH: u32 = 2;
const TOWERS_TO_SPAWN_BOSS: u32 = 10; //10
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TowerBuiltEvent>()
            .add_event::<TowerRemoveEvent>()
            .add_event::<PlaceTowerPreviewEvent>()
            .add_event::<SpawnBulletEvent>()
            //.add_system(spawn_tower)
            //.add_system(tower_input)
            .insert_resource(TowerSpawnCost { cost: 5 })
            .insert_resource(TowerCount {
                count: 0,
                boss_spawned: false,
            })
            .add_system(tower_mouse_input)
            .add_system(tower_key_input)
            .add_system(spawn_tower_preview)
            .add_system(preview_paid_for)
            .add_system(remove_tower)
            .add_system(tower_shoot)
            .add_system(spawn_bullet)
            .add_system(tick_bullet)
            .add_system(move_bullet)
            .add_system(bullet_hit);
        //.add_system(rotate_sprite);
    }
}

#[derive(Component)]
pub struct Tower {
    pub coords: HexCoords,
    pub refund: u32,
    shoot_timer: Timer,
    can_shoot: bool,
    range: f32,
}

impl Tower {
    pub fn new(coords: HexCoords, refund: u32) -> Self {
        Tower {
            coords,
            refund,
            shoot_timer: Timer::from_seconds(1.0, true),
            can_shoot: true,
            range: 200.0,
        }
    }
}

#[derive(Component)]
pub struct TowerPreview {}

#[derive(Bundle)]
pub struct PreviewTowerBundle {
    pub preview: TowerPreview,
    pub pile: GoldPile,
}

#[derive(Component)]
struct TowerSprite;

struct PlaceTowerPreviewEvent {
    //position: Vec3,
    coords: HexCoords,
}

// successfully build
pub struct TowerBuiltEvent {
    pub coords: HexCoords,
}

struct TowerRemoveEvent {
    coords: HexCoords,
}

struct TowerSpawnCost {
    cost: u32,
}

struct TowerCount {
    count: u32,
    boss_spawned: bool,
}

// fn rotate_sprite(
//     mut q_tower: Query<&mut Transform, With<Tower>>,
//     time: Res<Time>,
// ) {
//     for mut tower in q_tower.iter_mut() {
//         // doesn't look very good
//         //tower.rotate_x(time.delta_seconds() * 2.0);
//         // rotating around z looks fine. Kinda looks like a spinning attack charge up

//         // this looks much better
//         let y = (time.seconds_since_startup() * 5.0).sin() as f32;
//         tower.scale = Vec3{x: 1.0, y: y, z: 1.0};
//         // how to look once/twice?
//         // Timer?
//         // add a component, run a timer, remove component?
//     }
// }

fn tower_mouse_input(
    mut ev_place_preview: EventWriter<PlaceTowerPreviewEvent>,
    q_selection: Query<&Hex, With<Selection>>,
    input: Res<Input<MouseButton>>,
    accept: Res<AcceptInput>,
) {
    if accept.0 && input.just_pressed(MouseButton::Left) {
        for hex in q_selection.iter() {
            ev_place_preview.send(PlaceTowerPreviewEvent {
                //position: trans.translation,
                coords: hex.coords,
            });
        }
    }
}

fn tower_key_input(
    mut ev_remove_tower: EventWriter<TowerRemoveEvent>,
    q_selection: Query<&Hex, With<Selection>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::X) {
        for hex in q_selection.iter() {
            ev_remove_tower.send(TowerRemoveEvent { coords: hex.coords });
        }
    }
}

// where a tower will be
// Still needs gold brought to it to build it
fn spawn_tower_preview(
    mut commands: Commands,
    mut ev_place_preview: EventReader<PlaceTowerPreviewEvent>,
    q_empty_hexes: Query<(Entity, &Hex), Or<(Without<Tower>, Without<GoldPile>)>>,
    asset_server: Res<AssetServer>,
    mut cost: ResMut<TowerSpawnCost>,
) {
    for ev in ev_place_preview.iter() {
        for (ent, hex) in q_empty_hexes.iter() {
            if ev.coords.eq(&hex.coords) {
                // empty hex exists
                commands
                    .entity(ent)
                    .insert_bundle(PreviewTowerBundle {
                        preview: TowerPreview {},
                        pile: GoldPile::new(cost.cost),
                    })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(SpriteBundle {
                                texture: asset_server.load("sprites/UnbuiltTower.png"),
                                // sprite: Sprite {
                                //     color: LIGHT_BLUE,
                                //     custom_size: Some(Vec2::new(20.0, 20.0)),
                                //     ..default()
                                // },
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
                            .insert(TowerSprite);
                    });

                // it is now a Hex, TowerPreview, GoldPile,
                // with a sprite child
                cost.cost += TOWER_COST_GROWTH;
            }
        }
    }
}

fn preview_paid_for(
    mut commands: Commands,
    mut ev_pile_cap: EventReader<PileCapEvent>,
    q_preview_towers: Query<(Entity, &Children, &Hex, &GoldPile), With<TowerPreview>>,
    mut q_child: Query<&mut Handle<Image>, With<TowerSprite>>,
    asset_server: Res<AssetServer>,
    mut tower_count: ResMut<TowerCount>,
    mut ev_boss: EventWriter<BossSpawnEvent>,
    mut ev_remove_pile: EventWriter<PileRemoveEvent>,
) {
    for ev in ev_pile_cap.iter() {
        for (ent, children, hex, pile) in q_preview_towers.iter() {
            if ev.coords == hex.coords {
                //println!("Upgrade {:?}", hex.coords);

                // change the color of the preview to a tower color
                for &child in children.iter() {
                    let sprite = q_child.get_mut(child);

                    // change the sprite of the preview tower sprite to the built tower
                    if let Ok(mut s) = sprite {
                        *s = asset_server.load("sprites/Tower.png");
                    }
                }

                commands
                    .entity(ent)
                    //.remove_children(children)
                    //.remove_bundle::<PreviewTowerBundle>()
                    .remove::<TowerPreview>()
                    .insert(Tower::new(ev.coords, (pile.gold_cap as f32 * 0.8) as u32))
                    .insert(GoldSpawner::new());

                if !tower_count.boss_spawned {
                    tower_count.count += 1;
                    if tower_count.count == TOWERS_TO_SPAWN_BOSS {
                        tower_count.boss_spawned = true;
                        ev_boss.send(BossSpawnEvent);
                    }
                }

                ev_remove_pile.send(PileRemoveEvent { coords: ev.coords });

                break;
            }
        }
    }
}

fn remove_tower(
    mut commands: Commands,
    mut ev_remove: EventReader<TowerRemoveEvent>,
    mut ev_spawn_gold: EventWriter<SpawnGoldEvent>,
    q_towers: Query<(
        Entity,
        &Children,
        &Transform,
        &Hex,
        Option<&GoldPile>,
        Option<&TowerPreview>,
        Option<&Tower>,
    )>,
    q_sprite: Query<Entity, With<TowerSprite>>,
    mut counter: ResMut<TowerCount>,
    mut cost: ResMut<TowerSpawnCost>,
    //mut q_child: Query<&mut Sprite>,
) {
    for ev in ev_remove.iter() {
        for (ent, children, trans, hex, _opt_pile, opt_preview, opt_tower) in q_towers.iter() {
            if ev.coords == hex.coords {
                let mut opt_count = 0;
                if opt_preview.is_some() {
                    opt_count += 1;
                }
                if opt_tower.is_some() {
                    opt_count += 1;
                }
                if opt_count == 0 {
                    //println!("No optionals");
                    break;
                }

                let mut pile_count = 0;

                // if let Some(pile) = opt_pile {
                //     pile_count = pile.count;

                // } else
                if let Some(tower) = opt_tower {
                    pile_count = tower.refund;
                }
                //println!("Pile count: {:?}", pile_count);

                for _ in 0..pile_count {
                    ev_spawn_gold.send(SpawnGoldEvent {
                        position: trans.translation,
                    });
                }

                for &child in children {
                    // despawn child if it has a TowerSprite component
                    if q_sprite.get(child).is_ok() {
                        commands.entity(child).despawn_recursive();
                    }
                }

                if opt_preview.is_some() {
                    commands.entity(ent).remove::<TowerPreview>();
                }

                if opt_tower.is_some() {
                    commands.entity(ent).remove::<GoldSpawner>();
                    commands.entity(ent).remove::<Tower>();

                    if !counter.boss_spawned {
                        // probably can't underflow
                        // can only destroy a tower if it exists
                        // but to be safe
                        if counter.count > 1 {
                            counter.count -= 1;
                        }
                    }
                    // likewise shouldn't need this check either
                    if cost.cost > TOWER_COST_GROWTH {
                        cost.cost -= TOWER_COST_GROWTH;
                    }
                }
            }
        }
    }
}

fn tower_shoot(
    mut q_towers: Query<(&Transform, &mut Tower)>,
    q_enemies: Query<(&Transform, &Enemy)>,
    mut ev_shoot: EventWriter<SpawnBulletEvent>,
    time: Res<Time>,
) {
    for (t_trans, mut t) in q_towers.iter_mut() {
        if t.can_shoot {
            // can shoot
            // find a target
            let direction = q_enemies
                .iter()
                .min_by_key(|target_transform| {
                    FloatOrd(Vec3::distance(
                        target_transform.0.translation,
                        t_trans.translation,
                    ))
                })
                .map(|closest_target| closest_target.0.translation - t_trans.translation);

            if let Some(direction) = direction {
                //println!("Shoot a bullet");
                // only shoot if within range
                if direction.length_squared() < (t.range * t.range) {
                    ev_shoot.send(SpawnBulletEvent {
                        pos: t_trans.translation.truncate(),
                        dir: direction.truncate(),
                    });
                    t.can_shoot = false;
                }
            }
        } else {
            // tick between shots when you can't shoot
            if t.shoot_timer.tick(time.delta()).just_finished() {
                t.can_shoot = true;
            }
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    dir: Vec2,
    timer: Timer,
}

impl Bullet {
    pub fn new(dir: Vec2) -> Self {
        Bullet {
            dir,
            timer: Timer::from_seconds(1.0, false),
        }
    }
}

struct SpawnBulletEvent {
    pos: Vec2,
    dir: Vec2,
}

fn spawn_bullet(
    mut commands: Commands,
    mut ev_spawn_bullet: EventReader<SpawnBulletEvent>,
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn_bullet.iter() {
        //println!("Spawn a bullet. pos: {:?}, dir: {:?}", ev.pos, ev.dir);
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("sprites/Missile.png"),
                sprite: Sprite {
                    // Flip the logo to the left
                    flip_x: { ev.dir.x > 0.0 },
                    // And don't flip it upside-down ( the default )
                    flip_y: false,
                    ..default()
                },
                // sprite: Sprite {
                //     color: PURPLE,
                //     custom_size: Some(Vec2::new(6.0, 6.0)),
                //     ..default()
                // },
                transform: Transform {
                    translation: Vec3 {
                        x: ev.pos.x,
                        y: ev.pos.y,
                        z: 0.5,
                    },
                    ..default()
                },
                ..default()
            })
            .insert(Bullet::new(ev.dir.normalize_or_zero()));
    }
}

fn tick_bullet(
    mut commands: Commands,
    mut q_bullet: Query<(Entity, &mut Bullet)>,
    time: Res<Time>,
) {
    for (ent, mut b) in q_bullet.iter_mut() {
        if b.timer.tick(time.delta()).just_finished() {
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn move_bullet(mut q_bullet: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut trans, b) in q_bullet.iter_mut() {
        trans.translation += b.dir.extend(0.0) * time.delta_seconds() * 400.0;
    }
}

pub fn bullet_hit(
    mut commands: Commands,
    q_bullet: Query<(Entity, &Transform), With<Bullet>>,
    q_enemies: Query<(Entity, &Transform), (Without<Bullet>, Without<Dead>, With<Enemy>)>,
) {
    for (b_ent, b_trans) in q_bullet.iter() {
        for (e_ent, e_trans) in q_enemies.iter() {
            if collide(
                b_trans.translation,
                Vec2::new(6., 6.),
                e_trans.translation,
                Vec2::new(15., 15.),
            )
            .is_some()
            {
                //println!("Blam!");
                // Todo drop gold
                commands.entity(e_ent).insert(Dead);

                commands.entity(b_ent).despawn_recursive();

                // println!("Grabbed a gold");
                // enemy.has_gold = true;
                // commands.entity(ent).remove::<Gold>();

                // commands.entity(e_ent).add_child(ent);
                // gold_trans.translation = Vec3::new(0.0, 0.0, 0.1);
                break;
            }
        }
    }
}
