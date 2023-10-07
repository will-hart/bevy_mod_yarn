// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_mod_yarn::{
    prelude::{BevyYarnEvent, BevyYarnStepDialogueEvent, YarnData},
    YarnPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, YarnPlugin::default()))
        .add_systems(Update, handle_yarn_steps)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn the yarn data file, starting the story
    commands.spawn(YarnData {
        yarnc_path: "../assets/minimal.yarnc".to_string(),
    });

    commands.spawn((TextBundle::from_section(
        "My story is below...\n\n",
        TextStyle {
            font_size: 16.,
            color: Color::WHITE,
            ..default()
        },
    ),));
}

fn handle_yarn_steps(
    mut events: EventReader<BevyYarnEvent>,
    mut event_sender: EventWriter<BevyYarnStepDialogueEvent>,
    mut texts: Query<&mut Text>,
) {
    for event in events.iter() {
        match event {
            BevyYarnEvent::Say(line) => {
                info!(
                    "Received a line from the yarn spinner engine, '{}'",
                    line.formatted_text
                );

                let mut text = texts.single_mut();
                text.sections.push(TextSection {
                    value: format!("{}\n", line.formatted_text),
                    style: TextStyle {
                        font_size: 16.,
                        color: Color::WHITE,
                        ..default()
                    },
                });

                event_sender.send(BevyYarnStepDialogueEvent);
            }
            BevyYarnEvent::Choices(_) | BevyYarnEvent::Command(_) => {
                warn!("Unexpected event for minimal example, ignoring. Event: {event:?}");
                event_sender.send(BevyYarnStepDialogueEvent);
            }
            BevyYarnEvent::EndConversation => {
                info!("Reached end of conversation, stopping");
            }
        }
    }
}
