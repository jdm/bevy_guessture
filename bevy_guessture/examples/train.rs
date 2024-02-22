use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_guessture::{GuessturePlugin, GestureRecord, GestureState, RecordedPath, GestureTemplates};
use guessture::{Template, find_matching_template_with_defaults};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Copy, Clone)]
enum RecordType {
    Template,
    Attempt,
}

#[derive(Default, Resource)]
struct RecordState {
    state: Option<RecordType>,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .init_resource::<RecordState>()
        .add_event::<VisiblePathEvent>()
        .add_event::<TextEvent>()
        .add_systems(Update, (
            recorded_path,
            keyboard_input,
            create_visible_path,
            fade_visible_path,
            update_text,
        ))
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gesture trainer".to_string(),
                canvas: Some("#bevy".to_owned()),
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GuessturePlugin::default())
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    commands.spawn((
        TextBundle::from_section(
            "Space: record a template\nShift: attempt a gesture\nEnter: save all templates\nO: load templates",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            }
        ).with_style(
            Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
        ),
    ));
}

#[derive(Event)]
struct VisiblePathEvent {
    path: Vec<(f32, f32)>,
    color: Color,
}

#[derive(Component)]
struct PathComponent;

#[derive(Component)]
struct MainCamera;

const RADIUS: f32 = 50.;

fn create_visible_path(
    mut events: EventReader<VisiblePathEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    for ev in events.read() {
        let (camera, camera_transform) = q_camera.single();
        for point in &ev.path {
            let Some(remapped) = camera.viewport_to_world_2d(
                camera_transform,
                Vec2::new(point.0, point.1),
            ) else {
                continue
            };

            commands.spawn((
                PathComponent,
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::new(RADIUS)).into(),
                    material: materials.add(ColorMaterial::from(ev.color)),
                    transform: Transform::from_translation(
                        Vec3::new(remapped.x, remapped.y, 0.),
                    ),
                    ..default()
                },
            ));
        }
    }
}

fn fade_visible_path(
    mut path: Query<(Entity, &mut Transform), With<PathComponent>>,
    mut commands: Commands,
) {
    for (entity, mut transform) in &mut path {
        transform.scale.x *= 0.9;
        transform.scale.y *= 0.9;
        if transform.scale.x < 0.01 {
            commands.entity(entity).despawn();
        }
    }
}

fn recorded_path(
    mut events: EventReader<RecordedPath>,
    mut state: ResMut<GestureState>,
    mut path_events: EventWriter<VisiblePathEvent>,
    mut record_state: ResMut<RecordState>,
) {
    for event in events.read() {
        match record_state.state.as_ref().unwrap() {
            RecordType::Attempt => {
                let matched_template = find_matching_template_with_defaults(
                    &state.templates,
                    &event.path,
                );
                match matched_template {
                    Ok((template, score)) if score >= 0.8 => {
                        println!("matched {} with score {}", template.name, score);
                        path_events.send(VisiblePathEvent {
                            color: Color::GREEN,
                            path: event.path.points(),
                        });
                    }
                    Ok((template, score)) => {
                        println!("matched {} but with score {}", template.name, score);
                    }
                    Err(err) => println!("failed to match: {:?}", err),
                }
            }

            RecordType::Template => {
                let Ok(template) = Template::new(
                    state.templates.len().to_string(),
                    &event.path,
                ) else {
                    continue;
                };
                println!("done recording template {}", template.name);
                state.templates.push(template);
                path_events.send(VisiblePathEvent {
                    color: Color::BLUE,
                    path: event.path.points(),
                });
            }
        }
        record_state.state = None;
    }
}

fn save_templates(state: &GestureState) -> Result<(), ()> {
    let serialized = state.serialize_templates()?;
    let path = Path::new("data.gestures");
    let mut f = File::create(path).map_err(|_| ())?;
    f.write(serialized.as_bytes()).map_err(|_| ())?;
    Ok(())
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut record_events: EventWriter<GestureRecord>,
    mut ui_events: EventWriter<TextEvent>,
    state: Res<GestureState>,
    server: Res<AssetServer>,
    mut record_state: ResMut<RecordState>,
) {
    if keys.just_pressed(KeyCode::ShiftLeft) {
        record_state.state = Some(RecordType::Attempt);
        record_events.send(GestureRecord::Start);
        ui_events.send(TextEvent::Show("Recording".to_owned()));
    }
    if keys.just_released(KeyCode::ShiftLeft) {
        record_events.send(GestureRecord::Stop);
        ui_events.send(TextEvent::Hide);
    }

    if keys.just_pressed(KeyCode::Space) {
        record_state.state = Some(RecordType::Template);
        record_events.send(GestureRecord::Start);
        ui_events.send(TextEvent::Show("Recording template".to_owned()));
    }
    if keys.just_released(KeyCode::Space) {
        record_events.send(GestureRecord::Stop);
        ui_events.send(TextEvent::Hide);
    }

    if keys.just_released(KeyCode::Enter) {
         match save_templates(&state) {
            Ok(()) => {
                ui_events.send(TextEvent::Show("Saved templates".to_owned()));
            }
            Err(()) => {
                ui_events.send(TextEvent::Show("Error saving templates".to_owned()));
            }
        }
    }

    if keys.just_released(KeyCode::KeyO) {
        let _handle: Handle<GestureTemplates> = server.load("data.gestures");
        ui_events.send(TextEvent::Show("Loading templates".to_owned()));
    }
}

#[derive(Event)]
enum TextEvent {
    Show(String),
    Hide,
}

#[derive(Component)]
struct UiText;

fn update_text(
    mut events: EventReader<TextEvent>,
    query: Query<Entity, With<UiText>>,
    mut commands: Commands,
) {
    for event in events.read() {
        for entity in &query {
            commands.entity(entity).despawn();
        }

        match event {
            TextEvent::Show(ref text) => {
                commands.spawn((
                    UiText,
                    TextBundle::from_section(
                        text.clone(),
                        TextStyle {
                            font_size: 50.0,
                            color: Color::GOLD,
                            ..default()
                        }
                    ).with_style(
                        Style {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(5.0),
                            left: Val::Px(15.0),
                            ..default()
                        },
                    ),
                ));
            }

            TextEvent::Hide => ()
        }
    }
}
