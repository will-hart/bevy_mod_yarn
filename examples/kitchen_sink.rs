// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_mod_yarn::{
    commands::AddBevyCommandHandlerExt,
    prelude::{BevyYarnEvent, BevyYarnStepDialogueEvent, YarnData},
    BevyYarnDialogueEngine, YarnPluginBuilder,
};

/// We use this resource to track dialogue state. This allows us to ignore invalid
/// options from keyboard input.
#[derive(Default, Resource)]
struct DialogueState {
    // The current number of choices - setting to 0 means there are no valid choices
    // and choice selection keys will be ignored in the `handle_input` system
    num_choices: usize,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Unlike the minimal example, we're using the plugin builder here to give us a
            // way to pre-register yarn commands.
            YarnPluginBuilder::default()
                // We can preregister one or more commands this way. This means that when your yarn file has
                // the command `<<set_background blah>>`, the rust `set_background` function will run
                .with_yarn_command("set_background", set_background)
                .build(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(DialogueState::default())
        // This is another way to register commands. This is also available on World.
        .add_yarn_command("echo", echo_handler)
        .add_systems(Update, (handle_yarn_steps, handle_input))
        .add_systems(Startup, setup)
        .run();
}

/// Pretty basic stuff here, we load the yarn file (note that you'll need to compile the file
/// and rename the file-Lines.csv and file-Metadata.csv files to `file.lines.csv` and
/// `file.metadata.csv` respectively. See build.rs for an example)
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn the yarn data file, starting the story
    commands.spawn(YarnData {
        yarnc_path: "../assets/kitchen_sink.yarnc".to_string(),
    });

    commands.spawn((TextBundle::from_section(
        "",
        TextStyle {
            font_size: 16.,
            color: Color::WHITE,
            ..default()
        },
    ),));
}

/// This system just takes user input and controls the dialogue engine using
/// that input. This doesn't do any checks (i.e. you can send options when there)
/// aren't any options to send, etc.
fn handle_input(
    keys: Res<Input<KeyCode>>,
    state: Res<DialogueState>,
    mut event_sender: EventWriter<BevyYarnStepDialogueEvent>,
    mut engines: Query<&mut BevyYarnDialogueEngine>,
) {
    for mut engine in engines.iter_mut() {
        if state.num_choices > 0 {
            if keys.just_pressed(KeyCode::Key1) || keys.just_pressed(KeyCode::Numpad1) {
                info!("Sending step event (option 1 pressed)");
                let _ = engine.vm.set_selected_option(0);
                event_sender.send(BevyYarnStepDialogueEvent);
            }

            if state.num_choices > 1
                && (keys.just_pressed(KeyCode::Key2) || keys.just_pressed(KeyCode::Numpad2))
            {
                info!("Sending step event (option 2 pressed)");
                let _ = engine.vm.set_selected_option(1);
                event_sender.send(BevyYarnStepDialogueEvent);
            }

            if state.num_choices > 2
                && (keys.just_pressed(KeyCode::Key3) || keys.just_pressed(KeyCode::Numpad3))
            {
                info!("Sending step event (option 3 pressed)");
                let _ = engine.vm.set_selected_option(2);
                event_sender.send(BevyYarnStepDialogueEvent);
            }
        } else if keys.just_pressed(KeyCode::Space) {
            info!("Sending step event (space pressed)");
            event_sender.send(BevyYarnStepDialogueEvent);
        }
    }
}

/// This function listens for BevyYarnEvents, which are sent by the Yarn Engine when
/// something new occurs within the dialogue. Here we can display the scene to the user.
fn handle_yarn_steps(
    mut state: ResMut<DialogueState>,
    mut events: EventReader<BevyYarnEvent>,
    mut texts: Query<&mut Text>,
) {
    for event in events.iter() {
        match event {
            BevyYarnEvent::Say(line) => {
                info!(
                    "Received a line from the yarn spinner engine, '{}'",
                    line.formatted_text
                );

                state.num_choices = 0;

                let mut text = texts.single_mut();
                text.sections.push(TextSection {
                    value: format!(
                        "{}{}\n",
                        if let Some(ref character) = line.character {
                            format!("{character} said: ")
                        } else {
                            "".into()
                        },
                        line.formatted_text
                    ),
                    style: TextStyle {
                        font_size: 16.,
                        color: Color::WHITE,
                        ..default()
                    },
                });
            }
            BevyYarnEvent::Choices(ref choices) => {
                let mut text = texts.single_mut();

                state.num_choices = choices.len();

                let section = TextSection {
                    value: choices
                        .iter()
                        .enumerate()
                        .map(|(item, choice)| {
                            format!("[{}] {}\n", item + 1, choice.formatted_line.formatted_text)
                        })
                        .collect::<String>(),
                    style: TextStyle {
                        font_size: 16.,
                        color: Color::RED,
                        ..default()
                    },
                };
                text.sections.push(section);
            }
            BevyYarnEvent::Command(cmd) => {
                state.num_choices = 0;

                // If bevy_mod_yarn handles the command using one of your pre-registered command handlers,
                // it will set `handled` to true. Otherwise its up to you to do something about these unhandled commands.
                if !cmd.handled {
                    warn!("Received an unexpected command: `{cmd:?}`. You should probably do something about it");
                }
            }
            BevyYarnEvent::EndConversation => {
                state.num_choices = 0;

                info!("Reached end of conversation, stopping");
            }
        }
    }
}

/// Used to mark a background graphic that is spawned by the `set_background` yarn command
#[derive(Component)]
pub struct BackgroundGraphic;

/// Called when a background is set from the yarn file, despawns previous
/// backgrounds and spawns new ones.
///
/// The purpose of this is to demonstrate pre-registered Yarn Spinner commands
/// that can be handled within your custom bevy app code. This is registered using
/// the plugin builder syntax in your app setup code.
pub(crate) fn set_background(world: &mut World, args: Vec<String>) {
    info!("Setting background with args {:?}", args);

    // despawn old backgrounds
    let backgrounds = world
        .query_filtered::<Entity, With<BackgroundGraphic>>()
        .iter(world)
        .collect::<Vec<_>>();

    for background in backgrounds.iter() {
        world.despawn(*background);
    }

    // spawn a new background
    world.resource_scope(|world, asset_server: Mut<AssetServer>| {
        for path in args.iter() {
            world.spawn((
                SpriteBundle {
                    texture: asset_server.load(format!("{}.png", path)),
                    ..Default::default()
                },
                BackgroundGraphic,
            ));
        }
    })
}

/// Another simpler handler that is registered on the bevy app directly,
/// as opposed to registering by using the plugin builder.
pub fn echo_handler(_world: &mut World, args: Vec<String>) {
    info!("ECHO: {args:?}");
}
