use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{ActiveEvents, CollisionGroups};
use bevy_seedling::prelude::*;

use crate::{
    camera::MainCamera,
    character_controller::CharacterController,
    health::Health,
    physics::{ENEMY_GROUP, EXPLOSION_GROUP, PLAYER_GROUP, WALL_GROUP},
    shotgun::{Shotgun, ShotgunViewModel},
    states::{GameState, PauseState},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state_scoped_event::<PlayerHurtEvent>(GameState::InGame)
            .add_state_scoped_event::<PlayerDeathEvent>(GameState::InGame)
            .add_systems(Update, on_player_death)
            .add_systems(
                Update,
                (update_player, handle_player_death)
                    .run_if(in_state(GameState::InGame).and(in_state(PauseState::Unpaused))),
            )
            .add_systems(OnEnter(GameState::InGame), spawn_player);
    }
}

#[derive(Debug, Component)]
#[require(
    Name::new("Player"),
    Health::new(100.0),
    Visibility::Visible,
    SpatialListener3D,
    CharacterController = CharacterController {
            max_speed: 15.0,
            acceleration: 10.0,
        },
    CollisionGroups::new(PLAYER_GROUP, ENEMY_GROUP | EXPLOSION_GROUP | WALL_GROUP),
    ActiveEvents::COLLISION_EVENTS,
    Shotgun,
)]
pub struct Player {
    pub invulnerability_timer: Timer,
    pub death_timer: Timer,
    pub dead: bool,
}

impl Default for Player {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.2, TimerMode::Once);
        // start the timer in a "finished" state
        timer.tick(Duration::from_secs_f32(1000.0));
        Player {
            invulnerability_timer: timer,
            death_timer: Timer::from_seconds(2.0, TimerMode::Once),
            dead: false,
        }
    }
}

impl Player {
    pub fn is_vulnerable(&self) -> bool {
        self.invulnerability_timer.finished()
    }
}

fn update_player(time: Res<Time>, mut query: Query<&mut Player>) {
    for mut player in query.iter_mut() {
        player.invulnerability_timer.tick(time.delta());
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player::default(), StateScoped(GameState::InGame)));
}

#[derive(Debug, Default, Clone, Copy, Event)]
pub struct PlayerDeathEvent;

fn handle_player_death(
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Single<(&Health, &mut Player)>,
    mut writer: EventWriter<PlayerDeathEvent>,
) {
    let (health, mut player) = player_query.into_inner();
    if !health.dead {
        return;
    }

    if !player.dead {
        writer.write(PlayerDeathEvent);
    }

    player.dead = true;

    player.death_timer.tick(time.delta());
    if player.death_timer.just_finished() {
        next_state.set(GameState::End);
    }
}

fn on_player_death(
    reader: EventReader<PlayerDeathEvent>,
    mut viewmodel_query: Query<&mut Visibility, With<ShotgunViewModel>>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    if reader.is_empty() {
        return;
    }

    // disable the view model
    for mut visibility in viewmodel_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }

    // drop the view to the ground
    camera.translation.y -= 0.25;
}

#[derive(Debug, Event)]
pub struct PlayerHurtEvent {}
