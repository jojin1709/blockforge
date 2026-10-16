use bevy::prelude::*;

pub const DAY_LENGTH: f32 = 360.0;

#[derive(Resource)]
pub struct DayNight {
    pub time: f32,
}

pub struct DayNightPlugin;

#[derive(Component)]
pub struct Sun;

impl Plugin for DayNightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DayNight { time: 0.25 })
            .add_systems(Update, tick_daynight);
    }
}

fn lerp_c(a: [f32; 3], b: [f32; 3], t: f32) -> Color {
    Color::srgb(
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    )
}

fn tick_daynight(
    time: Res<Time>,
    pause: Res<crate::pause::PauseOpen>,
    mut dn: ResMut<DayNight>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun_q: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut fog_q: Query<&mut DistanceFog>,
) {
    if pause.0 {
        return;
    }
    dn.time = (dn.time + time.delta_secs() / DAY_LENGTH) % 1.0;
    let angle = dn.time * std::f32::consts::TAU;
    let sun_dir = Vec3::new(angle.cos(), angle.sin(), 0.35).normalize();
    let daylight = sun_dir.y.clamp(0.0, 1.0);

    for (mut tf, mut light) in sun_q.iter_mut() {
        tf.translation = sun_dir * 150.0;
        tf.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -sun_dir);
        light.illuminance = 500.0 + 13_500.0 * daylight;
        light.shadow_maps_enabled = daylight > 0.05;
    }

    let day = [0.45, 0.70, 0.95];
    let sunset = [0.88, 0.48, 0.28];
    let night = [0.015, 0.02, 0.06];
    let y = sun_dir.y;
    let sky = if y > 0.18 {
        lerp_c(sunset, day, ((y - 0.18) / 0.25).min(1.0))
    } else if y > -0.08 {
        let k = (y + 0.08) / 0.26;
        lerp_c(night, sunset, k)
    } else {
        lerp_c(night, night, 0.0)
    };
    clear.0 = sky;
    ambient.brightness = 80.0 + 320.0 * daylight;
    for mut fog in fog_q.iter_mut() {
        fog.color = sky;
    }
}
