# bevy_guessture

This library integrates the `guessture` library into the Bevy ecosystem. Its responsibilities include:
* recording mouse position data in response to app-initiated events
* providing mouse path data for a completed recording window to the app
* storing app-accessible gesture templates
* exposing gesture template serialization and asset loading mechanisms

Bevy apps using `bevy_guessture` are responsible for setting up gesture templates,
triggering recording windows, and initiating gesture matching with the recorded mouse path data.
There is an example app that demonstrates visual integration of gesture recognition, as well as
serializing gesture information as a loadable asset.

To get started, install the `GuessturePlugin` in your app and prepare a set of guesture templates:
```rs
    App::new()
        .add_plugins(GuessturePlugin::default());
```
Then prepare a set of gesture templates:
```rs
fn setup(server: Res<AssetServer>) {
    let _handle: Handle<GestureTemplates> = server.load("data.gestures");
}
```

To start recording a potential gesture, send the appropriate event:
```rs
fn start_record(mut record_events: EventWriter<GestureRecord>) {
   record_events.send(GestureRecord::Start);
}
```

After later sending a `GestureRecord::Stop` event, wait for a `RecordedPath` event with the complete recording:
```rs
fn recorded_path(
    mut events: EventReader<RecordedPath>,
    mut state: ResMut<GestureState>,
) {
    let matched_template = find_matching_template_with_defaults(
        &state.templates,
        &event.path,
    );
    match matched_template {
        Ok((template, score)) =>
            println!("matched {} with score {}", template.name, score),
        Err(err) =>
            println!("failed to match: {:?}", err),
    }
}
```
