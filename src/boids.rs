use crate::{
    enemies::{move_enemies, Enemy},
    gold::{Gold, MouseFollow},
};
use bevy::prelude::*;
use rand::prelude::*;

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(boids_gold)
            .add_system(boids_enemy)
            .add_system(move_boids.before(move_enemies));
    }
}

#[derive(Component)]
pub struct Boid {
    sep_dir: Vec2,
}

impl Boid {
    pub fn new() -> Self {
        Boid {
            sep_dir: Vec2::ZERO,
        }
    }
}

fn boids_gold(mut q_gold: Query<(&Transform, &mut Boid), (Without<MouseFollow>, With<Gold>)>) {
    // can't double loop the same query
    let mut combinations = q_gold.iter_combinations_mut();
    while let Some([a, b]) = combinations.fetch_next() {
        // mutably access components data

        // ignore z
        let (pos_a, mut boid_a) = (a.0.translation.truncate(), a.1);
        let (pos_b, mut boid_b) = (b.0.translation.truncate(), b.1);

        let d = pos_a.distance(pos_b);
        if d < 10.0 {
            let mut dir = pos_a - pos_b; // dir away
                                         // could've just been d<0.1
            if dir.x.abs() < 0.1 && dir.y.abs() < 0.1 {
                // close enough to 0
                let mut rng = rand::thread_rng();
                dir = Vec2::new(rng.gen(), rng.gen());
            }
            dir = dir.normalize_or_zero() * (d + 0.01); // for when d = 0

            boid_a.sep_dir += dir;
            boid_b.sep_dir -= dir;
        }
    }
}

fn boids_enemy(mut q_enemy: Query<(&Transform, &mut Boid), With<Enemy>>) {
    let mut combinations = q_enemy.iter_combinations_mut();
    while let Some([a, b]) = combinations.fetch_next() {
        // mutably access components data

        // ignore z
        let (pos_a, mut boid_a) = (a.0.translation.truncate(), a.1);
        let (pos_b, mut boid_b) = (b.0.translation.truncate(), b.1);

        let d = pos_a.distance(pos_b);
        if d < 10.0 {
            let mut dir = pos_a - pos_b; // dir away

            if d < 0.01 {
                // close enough to 0
                let mut rng = rand::thread_rng();
                dir = Vec2::new(rng.gen(), rng.gen());
            }
            dir = dir.normalize_or_zero() * (d + 0.01); // for when d = 0

            boid_a.sep_dir += dir;
            boid_b.sep_dir -= dir;
        }
    }
}

fn move_boids(mut q_boid: Query<(&mut Transform, &mut Boid)>, time: Res<Time>) {
    for (mut trans, mut boid) in q_boid.iter_mut() {
        trans.translation +=
            boid.sep_dir.normalize_or_zero().extend(0.0) * time.delta_seconds() * 60.0;
        boid.sep_dir = Vec2::ZERO;
    }
}
