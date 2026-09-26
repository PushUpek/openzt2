use bevy::prelude::*;

pub(super) fn show_missing_game_content_screen(error: &std::io::Error) {
    let mut application = App::new();
    application
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenZT2 — game files unavailable".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK));
    application.world_mut().spawn(Camera2d);
    application.world_mut().spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(32),
            right: px(32),
            top: px(32),
            ..default()
        },
        Text::new(format!(
            "OpenZT2 could not load the required game files.\n\n\
             Place the OpenZT2 executable in your existing Zoo Tycoon 2 installation folder, \
             next to {}. Then start OpenZT2 again.\n\n\
             If you set OPENZT2_Z2F_PATH, it must point to that folder.\n\n{error}",
            z2f::REQUIRED_ARCHIVES.join(", "),
        )),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
    ));
    application.run();
}
