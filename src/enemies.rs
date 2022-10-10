use bevy::sprite::collide_aabb::collide;
use bevy::utils::Duration;
use bevy::{prelude::*, utils::FloatOrd};
use rand::prelude::*;

use crate::boids::Boid;
use crate::gold::GoldPile;
use crate::tower::bullet_hit;
use crate::StartSpawningEnemiesEvent;
use crate::{gold::Gold, palette::*};

const ENEMY_SPAWN_TIME: f32 = 10.0;
const BOSS_HEALTH: u32 = 750; //1000

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>()
            .add_event::<BossSpawnEvent>()
            .add_event::<BossCapEvent>()
            .insert_resource(EnemySpawnInfo { group_size: 10 })
            .add_system(setup)
            .add_system(generate_enemies)
            .add_system(spawn_enemy)
            .add_system(move_enemies)
            .add_system(move_shadow.after(move_enemies))
            .add_system(grab_gold.before(bullet_hit))
            .add_system(escape)
            .add_system(spawn_boss)
            // bullet_hit adds Dead. Run before it so it runs next frame
            // and then this entity won't be added to any other queries
            // what's the pattern?
            // run die code before the thing that sets Dead?
            .add_system(drop_gold_and_die.before(bullet_hit));
    }
}

#[derive(Component)]
pub struct Enemy {
    has_gold: bool,
}

impl Enemy {
    fn new() -> Self {
        Enemy { has_gold: false }
    }
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
struct EnemySpawner {
    timer: Timer,
}

#[derive(Component)]
pub struct Boss;

pub struct BossSpawnEvent;
pub struct BossCapEvent;

struct SpawnEnemyEvent {
    position: Vec3,
}

struct EnemySpawnInfo {
    group_size: u32,
}

fn setup(mut commands: Commands, mut ev_start: EventReader<StartSpawningEnemiesEvent>) {
    for _ev in ev_start.iter() {
        commands.spawn().insert(EnemySpawner {
            timer: Timer::new(Duration::from_secs_f32(ENEMY_SPAWN_TIME), true),
        });
    }
}

fn generate_enemies(
    time: Res<Time>,
    mut ev_spawn_enemy: EventWriter<SpawnEnemyEvent>,
    mut q_spawner: Query<&mut EnemySpawner>,
    mut info: ResMut<EnemySpawnInfo>,
) {
    for mut spawner in q_spawner.iter_mut() {
        if spawner.timer.tick(time.delta()).finished() {
            for _ in 0..info.group_size {
                let mut rng = rand::thread_rng();
                let spawn_pos = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
                    .normalize_or_zero()
                    * 500.;

                ev_spawn_enemy.send(SpawnEnemyEvent {
                    position: spawn_pos.extend(0.3),
                })
            }
            info.group_size += 1;
        }
    }
}

fn spawn_enemy(mut commands: Commands, mut ev_spawn_enemy: EventReader<SpawnEnemyEvent>) {
    for ev in ev_spawn_enemy.iter() {
        let e = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: CRIMSON,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform {
                    translation: ev.position,
                    ..default()
                },
                ..default()
            })
            .insert(Enemy::new())
            .insert(Boid::new())
            .id();

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform {
                    translation: ev.position,
                    ..default()
                },
                ..default()
            })
            .insert(Follow {
                target: e,
                offset: Vec3::new(1.0, -3.5, -0.1),
            });
    }
}

// can I do this in a different way to have things be able to chain follow?
// 1 query, optFollow?
fn move_shadow(
    mut commands: Commands,
    q_trans: Query<&Transform, Without<Follow>>,
    mut q_shadow: Query<(Entity, &mut Transform, &Follow)>,
) {
    for (ent, mut trans, f) in q_shadow.iter_mut() {
        let target = q_trans.get(f.target);
        match target {
            Ok(t) => {
                trans.translation = t.translation + f.offset;
            }
            Err(_) => {
                // target no longer exists
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
struct Follow {
    target: Entity,
    offset: Vec3,
}

fn spawn_boss(
    mut commands: Commands,
    mut ev_boss_spawn: EventReader<BossSpawnEvent>,
    asset_server: Res<AssetServer>,
) {
    for _ev in ev_boss_spawn.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("sprites/Monster.png"),
                transform: Transform {
                    translation: Vec3::new(400.0, 20.0, 0.2),
                    ..default()
                },
                ..default()
            })
            .insert(Boss)
            .insert(GoldPile {
                count: 0,
                gold_cap: BOSS_HEALTH,
            });
    }
}

pub fn move_enemies(
    mut q_enemies: Query<(&mut Transform, &Enemy), Without<Dead>>,
    q_gold: Query<&Transform, (With<Gold>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut trans, enemy) in q_enemies.iter_mut() {
        let mut dir = Vec3::new(0.0, 0.0, 0.0) - trans.translation;

        if enemy.has_gold {
            dir = trans.translation - Vec3::ZERO;
        } else {
            let direction = q_gold
                .iter()
                .min_by_key(|target_transform| {
                    FloatOrd(Vec3::distance(
                        target_transform.translation,
                        trans.translation,
                    ))
                })
                .map(|closest_target| closest_target.translation - trans.translation);

            if let Some(direction) = direction {
                dir = direction;
            }
        }

        dir.z = 0.0;

        trans.translation += dir.normalize_or_zero() * 100. * time.delta_seconds();
    }
}

// don't do it this way again
// despawn the gold
// add a sprite
// die and spawn a gold on the corpse
fn grab_gold(
    mut commands: Commands,
    mut q_enemies: Query<(Entity, &Transform, &mut Enemy), Without<Dead>>,
    mut q_gold: Query<(Entity, &mut Transform), (Without<Enemy>, With<Gold>)>,
) {
    // when you grab the gold, run away
    // directly away from 0,0 ?
    // remove the gold?
    // add something to the enemy so they don't pick up more gold?
    for (ent, mut gold_trans) in q_gold.iter_mut() {
        for (e_ent, e_trans, mut enemy) in q_enemies.iter_mut() {
            if enemy.has_gold {
                // don't pick up more gold
                // could've been an insert and Without<CarryingGold>
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
                //println!("Grabbed a gold: ent: {:?}", ent);
                enemy.has_gold = true;
                commands.entity(ent).remove::<Gold>();

                //println!("Add Child. e_ent: {:?}", e_ent);
                // here
                // so is the issue:
                // commands get added
                // then they get run at the end of frame
                // if the order is wrong, delete happens before add_child
                // so this needs to run before delete
                // would events help?
                // when hit, send an event to delete the enemy
                // probably not
                commands.entity(e_ent).add_child(ent);
                gold_trans.translation = Vec3::new(0.0, 0.0, 0.1);
                break;
            }
        }
    }
}

fn drop_gold_and_die(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Enemy, &Transform, Option<&Children>), Added<Dead>>,
    mut q_child: Query<(&mut Transform, &Sprite), Without<Enemy>>,
) {
    for (ent, enemy, e_trans, children) in q_enemies.iter() {
        if enemy.has_gold {
            if let Some(children) = children {
                // print!("Has some children");
                // println!(" len: {:?}", children.len());
                for &child in children.iter() {
                    //commands.entity(ent).remove_children(child);
                    //println!("Adding gold. len: {:?}", children.len());

                    // print!("Add Gold ");
                    // println!("child ent: {:?}", child);
                    // it's probably immediately colliding with gold
                    commands.entity(child).insert(Gold);
                    let child_trans = q_child.get_mut(child);
                    match child_trans {
                        Ok(mut t) => {
                            t.0.translation = e_trans.translation;
                            t.0.translation.z = 0.3;
                        }
                        Err(e) => {
                            error!("Error getting child transform: {e}");
                        }
                    }
                }
                // print!("Remove children");
                // println!(" ent: {:?}", ent);
                commands.entity(ent).remove_children(children);
                // pos is (0, 0)
            }
        }
        // kill enemy
        // print!("Kill Enemy ");
        // println!("ent: {:?}", ent);
        // Kill Enemy ent: 217v0
        // thread 'main' panicked at 'Entity 217v0 does not exist'
        // did a bullet hit the leftover gold?
        // maybe rapier_2d has logging for this type of thing
        commands.entity(ent).despawn();
    }
}

fn escape(mut commands: Commands, q_enemies: Query<(Entity, &Enemy, &Transform), Without<Dead>>) {
    for (ent, enemy, trans) in q_enemies.iter() {
        if enemy.has_gold && trans.translation.distance(Vec3::ZERO) > 700.0 {
            // escaped
            println!("Escaped");
            commands.entity(ent).despawn_recursive();
        }
    }
}
