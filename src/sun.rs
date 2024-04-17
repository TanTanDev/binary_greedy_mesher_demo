use std::time::Duration;

use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

pub const DAY_TIME_SEC: f32 = 60.0;
pub const NIGHT_TIME_SEC: f32 = 1.0;
pub const CYCLE_TIME: f32 = DAY_TIME_SEC + NIGHT_TIME_SEC;

///! current time of day
#[derive(Resource)]
struct SkyTime(pub f32);

// ticked update of skytime
#[derive(Resource)]
struct CycleTimer(Timer);

#[derive(Resource, Reflect)]
pub struct SunSettings {
    illuminance: f32,
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
pub struct Sun;

pub struct SunPlugin;

impl Plugin for SunPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SunSettings>();
        app.insert_resource(SkyTime(0f32));
        app.insert_resource(SunSettings {
            illuminance: 4000.0,
        });
        app.insert_resource(CycleTimer(Timer::new(
            Duration::from_millis(450),
            TimerMode::Repeating,
        )));
        app.add_systems(Update, daylight_cycle);
        app.add_plugins(ResourceInspectorPlugin::<SunSettings>::default());
    }
}

fn daylight_cycle(
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<SkyTime>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    sun_settings: Res<SunSettings>,
    mut cycle_timer: ResMut<CycleTimer>,
) {
    cycle_timer.0.tick(time.delta());

    if !cycle_timer.0.just_finished() {
        return;
    }
    let multiplier = if keyboard.pressed(KeyCode::KeyI) {
        6.0
    } else {
        1.0
    };
    // timer.0 += time.delta_seconds() * multiplier;
    timer.0 += cycle_timer.0.duration().as_secs_f32() * multiplier;
    if timer.0 > CYCLE_TIME {
        timer.0 -= CYCLE_TIME;
    }

    let day = (timer.0 / DAY_TIME_SEC).min(1.0);
    let night = ((timer.0 - DAY_TIME_SEC) / NIGHT_TIME_SEC).max(0.0);
    let percent = day * std::f32::consts::PI + night * std::f32::consts::PI;

    for (mut light_trans, mut directional) in query.iter_mut() {
        light_trans.rotation = Quat::from_rotation_x(-percent.sin().atan2(percent.cos()));
        directional.illuminance = percent.sin().max(0.0).powf(2.0) * sun_settings.illuminance;
    }
}
