use bevy::{
    app::{App, Startup, Update},
    asset::{AssetMode, AssetPlugin},
    ecs::query,
    math::{
        bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
        vec2,
    },
    prelude::*,
    render::{camera::Viewport, primitives::Aabb},
};
use rand::Rng;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    Playing,
    GameOver,
}

const PIPE_SPACE: f32 = 42.;
const PIPE_TO_PIPE_SPACE: f32 = 160.;
const PIPE_WIDTH: f32 = 26.;
const SCROLL_SPEED: f32 = -100.;
const TERMINAL_VELOCITY: f32 = -400.;
const JUMP_VELOCITY: f32 = 200.;
const GRAVITY: f32 = -982.;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Animation {
    t: f32,
    repeat: bool,
    frame: usize,
    frames: Vec<Frame>,
}

struct Frame {
    index: usize,
    duration: f32,
}

#[derive(Event, Default)]
struct OnJumped;

#[derive(Component)]
struct Velocity(f32);

#[derive(Resource)]
struct Gravity(f32);

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Collider(Aabb2d);

#[derive(Component)]
struct Root;

enum Atlas {
    Background = 0,
    Bird1 = 1,
    Bird2 = 2,
    Bird3 = 3,
    PipeTop = 4,
    PipeBottom = 5,
}

fn random_pipe_height() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(48..=154) as f32
}

fn startup(mut commands: Commands) {
    commands.insert_resource(Gravity(GRAVITY));
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 0.5,
            ..default()
        },
        camera: Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(288, 512),
                ..default()
            }),
            ..default()
        },
        ..default()
    });
}

fn create_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<Entity, With<Root>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }

    let flappy_sheet = asset_server.load::<Image>("flappy.png");

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(x, y, x + w, y + h)
    }

    let mut texture_atlas = TextureAtlasLayout::new_empty(vec2(433., 260.));
    // The background
    texture_atlas.add_texture(rect(3., 0., 144., 256.));
    // The first bird animation
    texture_atlas.add_texture(rect(381., 187., 16., 12.));
    // The second bird animation
    texture_atlas.add_texture(rect(381., 187. + 26., 16., 12.));
    // The third bird animation
    texture_atlas.add_texture(rect(381., 187. + 26. * 2., 16., 12.));
    // The top pipe
    texture_atlas.add_texture(rect(152., 3., PIPE_WIDTH, 160.));
    // The bottom pipe
    texture_atlas.add_texture(rect(180., 3., PIPE_WIDTH, 160.));

    let handle_texture_atlas = texture_atlases.add(texture_atlas);

    let bird_frames = vec![
        Frame {
            index: Atlas::Bird3 as usize,
            duration: 0.2,
        },
        Frame {
            index: Atlas::Bird2 as usize,
            duration: 0.2,
        },
        Frame {
            index: Atlas::Bird1 as usize,
            duration: 0.2,
        },
    ];

    commands
        .spawn((Root, SpatialBundle::default()))
        .with_children(|parent| {
            parent.spawn((
                Player,
                Collider(Aabb2d::new(Vec2::new(0., 0.), Vec2::new(6., 4.))),
                Velocity(0.),
                Animation {
                    frame: 2,
                    repeat: false,
                    t: 0.,
                    frames: bird_frames,
                },
                SpriteSheetBundle {
                    texture: flappy_sheet.clone(),
                    atlas: TextureAtlas {
                        layout: handle_texture_atlas.clone(),
                        index: Atlas::Bird1 as usize,
                    },
                    transform: Transform::from_translation(Vec3::new(0., 0., 4.)),
                    ..default()
                },
            ));

            parent
                .spawn((
                    Background,
                    SpriteSheetBundle {
                        texture: flappy_sheet.clone(),
                        atlas: TextureAtlas {
                            layout: handle_texture_atlas.clone(),
                            index: Atlas::Background as usize,
                        },
                        transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((SpriteSheetBundle {
                        texture: flappy_sheet.clone(),
                        atlas: TextureAtlas {
                            layout: handle_texture_atlas.clone(),
                            index: Atlas::Background as usize,
                        },
                        transform: Transform::from_translation(Vec3::new(143., 0., 0.)),
                        ..default()
                    },));
                });

            for i in 0..4 {
                let offset = random_pipe_height();
                parent
                    .spawn((
                        Obstacle,
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                i as f32 * PIPE_TO_PIPE_SPACE + 144.,
                                offset,
                                1.,
                            )),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Pipe,
                            Collider(Aabb2d::new(
                                Vec2::new(0., 0.),
                                Vec2::new(PIPE_WIDTH / 2., 80.),
                            )),
                            SpriteSheetBundle {
                                texture: flappy_sheet.clone(),
                                atlas: TextureAtlas {
                                    layout: handle_texture_atlas.clone(),
                                    index: Atlas::PipeTop as usize,
                                },
                                ..default()
                            },
                        ));
                        parent.spawn((
                            Pipe,
                            Collider(Aabb2d::new(
                                Vec2::new(0., 0.),
                                Vec2::new(PIPE_WIDTH / 2., 80.),
                            )),
                            SpriteSheetBundle {
                                texture: flappy_sheet.clone(),
                                atlas: TextureAtlas {
                                    layout: handle_texture_atlas.clone(),
                                    index: Atlas::PipeBottom as usize,
                                },
                                transform: Transform::from_translation(Vec3::new(
                                    0.,
                                    -160. - PIPE_SPACE,
                                    0.,
                                )),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn input(
    mut query: Query<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut writer: EventWriter<OnJumped>,
) {
    let mut velocity = query.single_mut();
    if buttons.just_pressed(MouseButton::Left) {
        velocity.0 = JUMP_VELOCITY;
        writer.send(OnJumped);
    }
}

fn apply_gravity(
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity) in &mut query {
        velocity.0 += gravity.0 * time.delta_seconds();
        velocity.0 = velocity.0.max(TERMINAL_VELOCITY);

        transform.translation.y += velocity.0 * time.delta_seconds();
    }
}

fn apply_rotation(mut query: Query<(&mut Transform, &Velocity), With<Player>>) {
    let (mut transform, velocity) = query.single_mut();

    // Make the player point towards the direction it's moving (up/down)
    let range = JUMP_VELOCITY - TERMINAL_VELOCITY;
    let normalized_velocity = (velocity.0 - TERMINAL_VELOCITY) / range;
    let rotation = (-90. + (normalized_velocity) * 180.0).clamp(-30., 90.);

    transform.rotation = transform.rotation.lerp(
        Quat::from_euler(EulerRot::YXZ, 0., 0., rotation.to_radians()),
        0.5,
    );
}

fn trigger_jump_animation(
    mut query: Query<&mut Animation, With<Player>>,
    mut reader: EventReader<OnJumped>,
) {
    let mut animation = query.single_mut();
    for _ in reader.read() {
        animation.frame = 0
    }
}

fn update_animation(
    mut query: Query<(&mut TextureAtlas, &mut Animation), With<Player>>,
    time: Res<Time>,
) {
    let mut delta = time.delta_seconds();

    for (mut texture_atlas, mut animation) in &mut query {
        loop {
            let frame = &animation.frames[animation.frame];

            let remaining = (1. - animation.t) * frame.duration;

            if delta < remaining {
                animation.t += delta / frame.duration;
                break;
            }

            delta -= remaining;

            let finished = animation.frame + 1 >= animation.frames.len();

            match (finished, animation.repeat) {
                (true, true) => {
                    animation.frame = 0;
                    animation.t = 0.;
                }
                (true, false) => {
                    animation.frame = animation.frames.len() - 1;
                    animation.t = 1.;
                    break;
                }
                _ => {
                    animation.frame += 1;
                    animation.t = 0.;
                }
            }
        }

        texture_atlas.index = animation.frames[animation.frame].index;
    }
}

// Eh, this should've been a material on a sprite
// but it's not implemented yet
fn scroll_backgrounds(mut query: Query<&mut Transform, With<Background>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation.x += time.delta_seconds() * SCROLL_SPEED;
        if transform.translation.x < -143. {
            transform.translation.x += 143.;
        }
    }
}

fn scroll_pipes(mut query: Query<&mut Transform, With<Obstacle>>, time: Res<Time>) {
    let scroll_back = PIPE_TO_PIPE_SPACE * 4.;
    for mut transform in &mut query {
        transform.translation.x += time.delta_seconds() * SCROLL_SPEED;
        if transform.translation.x < -144. * 2. {
            let offset = random_pipe_height();
            transform.translation.x += scroll_back;
            transform.translation.y = offset;
        }
    }
}

fn crash_and_die(
    mut query: Query<(&Transform, &Collider, &mut Velocity), With<Player>>,
    pipes: Query<(&GlobalTransform, &Collider), With<Pipe>>,
    mut state: ResMut<NextState<AppState>>,
) {
    let (transform, Collider(player_collider), mut velocity) = query.single_mut();

    let player = offset_aabb(player_collider, &transform.translation);

    if transform.translation.y < -128. || transform.translation.y > 128. {
        state.set(AppState::GameOver);
        velocity.0 = JUMP_VELOCITY * 2.;
        return;
    }

    for (t, Collider(pipe_collider)) in &pipes {
        let pipe = offset_aabb(pipe_collider, &t.translation());
        if pipe.intersects(&player) {
            state.set(AppState::GameOver);
            velocity.0 = JUMP_VELOCITY * 2.;
            return;
        }
    }
}

fn offset_aabb(aabb: &Aabb2d, translation: &Vec3) -> Aabb2d {
    let offset = translation.xy();
    Aabb2d::new(offset, aabb.half_size())
}

fn start_game(
    mut state: ResMut<NextState<AppState>>,
    mut query: Query<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut writer: EventWriter<OnJumped>,
) {
    let mut velocity = query.single_mut();
    if buttons.just_pressed(MouseButton::Left) {
        state.set(AppState::Playing);
        velocity.0 = JUMP_VELOCITY;
        writer.send(OnJumped);
    }
}

fn restart_game(mut state: ResMut<NextState<AppState>>, buttons: Res<ButtonInput<MouseButton>>) {
    if buttons.just_pressed(MouseButton::Left) {
        state.set(AppState::MainMenu);
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_state(AppState::MainMenu)
        .add_event::<OnJumped>()
        .add_systems(Startup, startup)
        .add_systems(OnEnter(AppState::MainMenu), create_world)
        .add_systems(Update, start_game.run_if(in_state(AppState::MainMenu)))
        .add_systems(Update, restart_game.run_if(in_state(AppState::GameOver)))
        .add_systems(
            Update,
            (apply_gravity, update_animation).run_if(not(in_state(AppState::MainMenu))),
        )
        .add_systems(
            Update,
            (
                input,
                trigger_jump_animation,
                scroll_backgrounds,
                scroll_pipes,
                crash_and_die,
                apply_rotation,
            )
                .run_if(in_state(AppState::Playing)),
        )
        .run();
}
