use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use guessture::{Path2D, Template};
use std::mem;

/// Plugin object to automatically integrate gesture recognition into your Bevy app.
#[derive(Default)]
pub struct GuessturePlugin;

impl Plugin for GuessturePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                JsonAssetPlugin::<GestureTemplates>::new(&["gestures"])
            )
            .add_systems(Update, (
                change_recording_state,
                update_templates,
                record_mouse
                    .run_if(|state: Res<GestureState>| state.current_recording.is_some())
            ))
            .add_event::<GestureRecord>()
            .add_event::<RecordedPath>()
            .init_resource::<GestureState>();
    }
}

/// A resource containing all gesture templates that will be considered.
/// Updating the `templates` member will affect all future match attempts.
#[derive(Default, Resource)]
pub struct GestureState {
    pub templates: Vec<Template>,
    current_recording: Option<Path2D>,
}

impl GestureState {
    /// Serialize all gesture templates as JSON. The result can be writtent
    /// to a `.gestures` file and subsequently loaded by Bevy as an asset.
    pub fn serialize_templates(&self) -> Result<String, ()> {
        let templates = GestureTemplates {
            templates: self.templates
                .iter()
                .map(|template| TemplateData {
                    name: template.name.clone(),
                    path: template.path.points(),
                })
                .collect(),
        };
        serde_json::to_string(&templates).map_err(|_| ())
    }
}

/// An event to toggle mouse path recording. Upon receiving a `Stop`
/// event, the plugin will send a corresponding [RecordedPath] event
/// containing the complete path since the original `Start` event was
/// received.
#[derive(Event)]
pub enum GestureRecord {
    Start,
    Stop,
}

/// An event following a [GestureRecord::Stop] event, containing a
/// complete path of points recorded from the mouse input.
#[derive(Event)]
pub struct RecordedPath {
    /// A 2d path of mouse positions. These can be passed immediately to
    /// the [guessture::find_matching_template] function to evaluate the
    /// path for known gestures.
    pub path: Path2D,
}

fn change_recording_state(
    mut events: EventReader<GestureRecord>,
    mut state: ResMut<GestureState>,
    mut path_event: EventWriter<RecordedPath>,
) {
    for event in events.iter() {
        match event {
            GestureRecord::Start => state.current_recording = Some(Path2D::default()),
            GestureRecord::Stop => {
                let Some(path) = mem::take(&mut state.current_recording) else { continue };
                path_event.send(RecordedPath {
                    path,
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

#[derive(serde::Deserialize, serde::Serialize, bevy::reflect::TypeUuid, bevy::reflect::TypePath)]
#[uuid = "502fa929-bfeb-52c4-9db0-4b8b380a2c46"]
pub struct GestureTemplates {
    templates: Vec<TemplateData>,
}

#[derive(serde::Deserialize, serde::Serialize)]
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
