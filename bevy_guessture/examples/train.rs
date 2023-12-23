use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_guessture::{GuessturePlugin, GestureRecord, GestureState, RecordedPath};
use guessture::{Template, find_matching_template_with_defaults};

#[derive(Copy, Clone)]
enum RecordType {
    Template,
    Attempt,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        //.insert_resource(MousePath::default())
        .add_event::<VisiblePathEvent>()
        .add_event::<TextEvent>()
        .add_systems(Update, (
            //cursor_position,
            recorded_path,
            keyboard_input,
            create_visible_path,
            fade_visible_path,
            update_text,
        ))
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy game".to_string(), // ToDo
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // The canvas size is constrained in index.html and build/web/styles.css
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GuessturePlugin::<RecordType>::new())
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

/*#[derive(Default, Resource)]
struct MousePath {
    path: Option<Path2D>,
    templates: Vec<Template>,
}

fn cursor_position(
    mut cursor_evr: EventReader<CursorMoved>,
    mut mouse_path: ResMut<MousePath>,
) {
    if let Some(ref mut path) = mouse_path.path {
        for ev in cursor_evr.iter() {
            let (x, y) = (ev.position.x, ev.position.y);
            if path.is_new_point(x, y) {
                path.push(x, y);
            }
        }
    }
}*/

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
    for ev in events.iter() {
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
                    mesh: meshes.add(shape::Circle::new(RADIUS).into()).into(),
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
    mut events: EventReader<RecordedPath<RecordType>>,
    mut state: ResMut<GestureState>,
    mut path_events: EventWriter<VisiblePathEvent>,
) {
    for event in events.iter() {
        match event.data {
            RecordType::Attempt => {
                let matched_template = find_matching_template_with_defaults(
                    &state.templates,
                    event.path.clone(),
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
                    event.path.clone(),
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
    }
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    //mut mouse_path: ResMut<MousePath>,
    mut record_events: EventWriter<GestureRecord<RecordType>>,
    mut ui_events: EventWriter<TextEvent>,
) {
    if keys.just_pressed(KeyCode::ShiftLeft) {
        //mouse_path.path = Some(Path2D::default());
        record_events.send(GestureRecord::Start);
        ui_events.send(TextEvent::Show("Recording".to_owned()));
    }
    if keys.just_released(KeyCode::ShiftLeft) {
        //let path = mem::take(&mut mouse_path.path).unwrap();
        //let points = path.points();
        //let matched_template = find_matching_template_with_defaults(&mouse_path.templates, path);
        record_events.send(GestureRecord::Stop(RecordType::Attempt));
        ui_events.send(TextEvent::Hide);
        /*match matched_template {
            Ok((template, score)) if score >= 0.8 => {
                println!("matched {} with score {}", template.name, score);
                events.send(VisiblePathEvent {
                    color: Color::GREEN,
                    path: points,
                });
            }
            Ok((template, score)) => {
                println!("matched {} but with score {}", template.name, score);
            }
            Err(err) => println!("failed to match: {:?}", err),
        }*/
    }

    if keys.just_pressed(KeyCode::Space) {
        //mouse_path.path = Some(Path2D::default());
        record_events.send(GestureRecord::Start);
        ui_events.send(TextEvent::Show("Recording template".to_owned()));        
    }
    if keys.just_released(KeyCode::Space) {
        record_events.send(GestureRecord::Stop(RecordType::Template));
        ui_events.send(TextEvent::Hide);
        /*let path = mem::take(&mut mouse_path.path).unwrap();
        let points = path.points();
        let Ok(template) = Template::new(mouse_path.templates.len().to_string(), path) else {
            return;
        };
        println!("done recording template {}", template.name);
        mouse_path.templates.push(template);
        events.send(VisiblePathEvent {
            color: Color::BLUE,
            path: points,
        });*/
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
    for event in events.iter() {
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
            TextEvent::Hide => {
                let entity = query.single();
                commands.entity(entity).despawn();
            }
        }
    }
}
