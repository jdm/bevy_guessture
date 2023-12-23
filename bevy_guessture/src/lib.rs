use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use guessture::{Path2D, Template};
use std::mem;
use std::marker::PhantomData;

pub struct GuessturePlugin<T> {
    _marker: PhantomData<T>,
}

impl <T: Clone + Send + Sync + 'static> GuessturePlugin<T> {
    pub fn new() -> GuessturePlugin<T> {
        GuessturePlugin {
            _marker: PhantomData,
        }
    }
}

impl <T: Clone + Send + Sync + 'static> Plugin for GuessturePlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                RonAssetPlugin::<GestureTemplates>::new(&["gestures"])
            )
            .add_systems(Update, (
                change_recording_state::<T>,
                update_templates,
                record_mouse
                    .run_if(|state: Res<GestureState>| state.current_recording.is_some())
            ))
            .add_event::<GestureRecord<T>>()
            .add_event::<RecordedPath<T>>()
            .init_resource::<GestureState>();
    }
}

#[derive(Default, Resource)]
pub struct GestureState {
    pub templates: Vec<Template>,
    current_recording: Option<Path2D>,
}

#[derive(Event)]
pub enum GestureRecord<T> {
    Start,
    Stop(T),
}

#[derive(Event)]
pub struct RecordedPath<T> {
    pub path: Path2D,
    pub data: T,
}

fn change_recording_state<T: Clone + Send + Sync + 'static>(
    mut events: EventReader<GestureRecord<T>>,
    mut state: ResMut<GestureState>,
    mut path_event: EventWriter<RecordedPath<T>>,
) {
    for event in events.iter() {
        match event {
            GestureRecord::Start => state.current_recording = Some(Path2D::default()),
            GestureRecord::Stop(data) => {
                let Some(path) = mem::take(&mut state.current_recording) else { continue };
                path_event.send(RecordedPath {
                    path,
                    data: data.clone(),
                });
            }
        }
    }
}

fn record_mouse(
    mut cursor_evr: EventReader<CursorMoved>,
    mut state: ResMut<GestureState>,
) {
    if let Some(ref mut path) = state.current_recording {
        for ev in cursor_evr.iter() {
            let (x, y) = (ev.position.x, ev.position.y);
            if path.is_new_point(x, y) {
                path.push(x, y);
            }
        }
    }
}

#[derive(serde::Deserialize, bevy::reflect::TypeUuid, bevy::reflect::TypePath)]
#[uuid = "502fa929-bfeb-52c4-9db0-4b8b380a2c46"]
pub struct GestureTemplates {
    templates: Vec<TemplateData>,
}

#[derive(serde::Deserialize)]
struct TemplateData {
    name: String,
    path: Vec<(f32, f32)>,
}

fn update_templates(
    mut ev_asset: EventReader<AssetEvent<GestureTemplates>>,
    mut state: ResMut<GestureState>,
    assets: Res<Assets<GestureTemplates>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let gestures = assets.get(handle).unwrap();
                for template_data in &gestures.templates {
                    let mut path = Path2D::default();
                    for &(x, y) in &template_data.path {
                        path.push(x, y);
                    }
                    let Ok(template) = Template::new_raw(
                        template_data.name.clone(), path
                    ) else {
                        continue
                    };
                    state.templates.push(template);
                }
            }

            AssetEvent::Modified { .. } | AssetEvent::Removed { .. } => continue,
        }
    }
}
