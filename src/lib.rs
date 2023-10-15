#![deny(missing_docs)]
//! A bevy helper for using the yarn dialogue format.

pub mod assets;
pub mod commands;
mod data;
mod events;

use std::collections::HashMap;

use assets::{
    BevyYarnMetadataTable, BevyYarnMetadataTableAssetLoader, BevyYarnProgram,
    BevyYarnProjectAssetLoader, BevyYarnStringTable, BevyYarnStringTableAssetLoader,
};
use bevy::prelude::*;
use commands::{BevyYarnCommand, CommandHandlers};
use data::YarnData;
use prelude::{
    BevyYarnChoice, BevyYarnEvent, BevyYarnLine, BevyYarnStepDialogueEvent, CommandHandlerFn,
};
use regex::Regex;
use yharnam::*;

use crate::assets::get_table_pathbuf_from_yarnc_path;

// TODO: allow setting locale
/// The locale to use for the yarn engine pluralisation etc
pub const LOCALE: &str = "en";

/// Core functionality of the crate
pub mod prelude {
    pub use crate::{
        assets::{BevyYarnMetadataTable, BevyYarnProgram, BevyYarnStringTable},
        commands::{BevyYarnCommand, CommandHandlerFn},
        data::{BevyYarnChoice, BevyYarnLine, YarnData},
        events::{BevyYarnEvent, BevyYarnStepDialogueEvent},
        BevyYarnDialogueEngine, YarnPlugin,
    };
}

/// A resource to contain the dialogue engine
#[derive(Component, Resource)]
pub struct BevyYarnDialogueEngine {
    /// The Yharnam virtual machine that runs the dialogue
    pub vm: VirtualMachine,

    /// The name of the file this engine was loaded from
    pub engine_name: String,

    /// The number of choices currently available to the user to select from
    pub num_choices: usize,

    /// A flag that is set to true to indicate that the dialogue is complete
    pub is_complete: bool,

    string_table: Handle<BevyYarnStringTable>,
    metadata_table: Handle<BevyYarnMetadataTable>,
    _program: Handle<BevyYarnProgram>,
}

/// A plugin that adds support for the Yarn engine
#[derive(Default)]
pub struct YarnPlugin {
    commands: Vec<(String, CommandHandlerFn)>,
}

impl Plugin for YarnPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<BevyYarnProgram>()
            .init_asset_loader::<BevyYarnProjectAssetLoader>()
            .add_asset::<BevyYarnStringTable>()
            .init_asset_loader::<BevyYarnStringTableAssetLoader>()
            .add_asset::<BevyYarnMetadataTable>()
            .init_asset_loader::<BevyYarnMetadataTableAssetLoader>()
            .add_event::<BevyYarnEvent>()
            .add_event::<BevyYarnStepDialogueEvent>()
            .insert_resource(CommandHandlers(HashMap::from_iter(self.commands.clone())))
            .add_systems(PreUpdate, (Self::load_yarn_data,))
            .add_systems(Update, (Self::process_yarn_events,));

        #[cfg(feature = "input-handlers")]
        app.add_systems(Update, (Self::handle_input,));
    }
}

impl YarnPlugin {
    /// A system that runs when a "yarn file" component is added and initialises the
    /// engine with the given data. Once the asset file is loaded, this system will
    /// remove the [`YarnData`] component and initialise a virtual machine.
    fn load_yarn_data(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        programs: Res<Assets<BevyYarnProgram>>,
        mut event_sender: EventWriter<BevyYarnStepDialogueEvent>,
        yarn_datas: Query<(Entity, &YarnData)>,
    ) {
        for (entity, data) in yarn_datas.iter() {
            let program_handle: Handle<BevyYarnProgram> = asset_server.load(&data.yarnc_path);

            if let Some(program) = programs.get(&program_handle) {
                let mut vm = VirtualMachine::new(program.program.clone());
                let string_table: Handle<BevyYarnStringTable> =
                    asset_server.load(get_table_pathbuf_from_yarnc_path(&data.yarnc_path, "lines"));
                let metadata_table: Handle<BevyYarnMetadataTable> = asset_server.load(
                    get_table_pathbuf_from_yarnc_path(&data.yarnc_path, "metadata"),
                );

                vm.set_node("Start").expect("set Start node");
                commands
                    .entity(entity)
                    .insert(BevyYarnDialogueEngine {
                        vm,
                        engine_name: data.yarnc_path.clone(),
                        _program: program_handle,
                        string_table,
                        metadata_table,
                        num_choices: 0,
                        is_complete: false,
                    })
                    .remove::<YarnData>();

                // trigger the first step
                info!("Finished loading program from {}", data.yarnc_path);
                event_sender.send(BevyYarnStepDialogueEvent);
            }
        }
    }

    /// Takes updates from the Yarn engine and forwards them to the ECS
    fn process_yarn_events(
        mut commands: Commands,
        string_tables: Res<Assets<BevyYarnStringTable>>,
        metadata_tables: Res<Assets<BevyYarnMetadataTable>>,
        command_handlers: Res<CommandHandlers>,
        mut read_step_events: EventReader<BevyYarnStepDialogueEvent>,
        mut send_yarn_events: EventWriter<BevyYarnEvent>,
        mut yarn_engines: Query<&mut BevyYarnDialogueEngine>,
    ) {
        for _ in read_step_events.iter() {
            debug!("Reading step event in process_yarn_events");

            for mut yarn_engine in yarn_engines.iter_mut() {
                let string_table = string_tables.get(&yarn_engine.string_table).unwrap();
                let metadata_table = metadata_tables.get(&yarn_engine.metadata_table).unwrap();

                loop {
                    match yarn_engine.vm.continue_dialogue() {
                        Ok(result) => {
                            match result {
                                SuspendReason::Nop => {}
                                SuspendReason::Line(line) => {
                                    yarn_engine.num_choices = 0;

                                    let (character, formatted_text) =
                                        string_table.get_final_text(&line, LOCALE);

                                    send_yarn_events.send(BevyYarnEvent::Say(BevyYarnLine {
                                        line: line.clone(),
                                        formatted_text,
                                        character,
                                        tags: metadata_table.get_tags_for_line(&line),
                                    }));
                                    break;
                                }
                                SuspendReason::Options(options) => {
                                    let choices = options
                                        .iter()
                                        .map(|choice| {
                                            let (character, formatted_text) =
                                                string_table.get_final_text(&choice.line, LOCALE);

                                            BevyYarnChoice {
                                                line_id: choice.line.id.clone(),
                                                formatted_line: BevyYarnLine {
                                                    formatted_text,
                                                    character,
                                                    tags: metadata_table
                                                        .get_tags_for_line(&choice.line),
                                                    line: choice.line.clone(),
                                                },
                                                destination_node: choice.destination_node.clone(),
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    yarn_engine.num_choices = choices.len();

                                    send_yarn_events.send(BevyYarnEvent::Choices(choices));
                                    break;
                                }
                                SuspendReason::Command(cmd_text) => {
                                    debug!("Received command {cmd_text}");
                                    yarn_engine.num_choices = 0;

                                    let command_parser =
                                        Regex::new(r#"(("[^"]+")|\S+)+"#).expect("parse regex");

                                    // parse the command name and args
                                    let (command_name, args) = command_parser
                                        .find_iter(&cmd_text)
                                        .map(|cap| cap.as_str().to_owned().replace('"', ""))
                                        .enumerate()
                                        .fold(
                                            (String::new(), Vec::<String>::new()),
                                            |mut acc, (index, item)| {
                                                if index == 0 {
                                                    (item, acc.1)
                                                } else {
                                                    acc.1.push(item);
                                                    acc
                                                }
                                            },
                                        );

                                    let mut bevy_command = BevyYarnCommand {
                                        command_name,
                                        args,
                                        handled: false,
                                    };

                                    // see if we have a handler registered
                                    if command_handlers.0.get(&bevy_command.command_name).is_some()
                                    {
                                        info!(
                                            "Calling registered command {} with args {:?}",
                                            bevy_command.command_name, bevy_command.args
                                        );
                                        bevy_command.handled = true;
                                        commands.add(bevy_command.clone());
                                    } else {
                                        info!(
                                            "Found unregistered command {} with args {:?}",
                                            bevy_command.command_name, bevy_command.args
                                        );
                                    }

                                    // raise an event either way
                                    send_yarn_events.send(BevyYarnEvent::Command(bevy_command));
                                }
                                SuspendReason::NodeChange { start, end } => {
                                    debug!("Move from node {start} to node {end}");
                                    yarn_engine.num_choices = 0;

                                    // do not break here as we want to trigger the first line of the next node
                                }
                                SuspendReason::DialogueComplete(last_node) => {
                                    debug!("End dialogue on {last_node}");
                                    yarn_engine.num_choices = 0;
                                    yarn_engine.is_complete = true;

                                    send_yarn_events.send(BevyYarnEvent::EndConversation);
                                    break;
                                }
                                SuspendReason::InvalidOption(option) => {
                                    warn!("Invalid option selected: {option}");
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Encountered error during yarn execution: {e:?}");
                        }
                    }
                }
            }
        }
    }

    #[cfg(feature = "input-handlers")]
    fn handle_input(
        keys: Res<Input<KeyCode>>,
        mut event_sender: EventWriter<BevyYarnStepDialogueEvent>,
        mut engines: Query<&mut BevyYarnDialogueEngine>,
    ) {
        for mut engine in engines.iter_mut() {
            if engine.num_choices > 0 {
                if keys.just_pressed(KeyCode::Key1) || keys.just_pressed(KeyCode::Numpad1) {
                    info!("Sending step event (option 1 pressed)");
                    let _ = engine.vm.set_selected_option(0);
                    event_sender.send(BevyYarnStepDialogueEvent);
                }

                if engine.num_choices > 1 && keys.just_pressed(KeyCode::Key2)
                    || keys.just_pressed(KeyCode::Numpad2)
                {
                    info!("Sending step event (option 2 pressed)");
                    let _ = engine.vm.set_selected_option(1);
                    event_sender.send(BevyYarnStepDialogueEvent);
                }

                if engine.num_choices > 2 && keys.just_pressed(KeyCode::Key3)
                    || keys.just_pressed(KeyCode::Numpad3)
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
}

/// Builds up a YarnPlugin with the given configuration
#[derive(Default)]
pub struct YarnPluginBuilder {
    commands: Vec<(String, CommandHandlerFn)>,
}

impl YarnPluginBuilder {
    /// Adds the given yarn command handlers to the builder, replacing any existing
    /// commands. Returns the builder.
    pub fn with_yarn_commands(mut self, yarn_commands: Vec<(String, CommandHandlerFn)>) -> Self {
        self.commands = yarn_commands;
        self
    }

    /// Adds a command to the command handlers, keeping the existing commands in place.
    /// Returns the builder
    pub fn with_yarn_command<N: Into<String>>(
        mut self,
        command_name: N,
        command: CommandHandlerFn,
    ) -> Self {
        self.commands.push((command_name.into(), command));
        self
    }

    /// Builds a yarn plugin
    pub fn build(self) -> YarnPlugin {
        YarnPlugin {
            commands: self.commands,
        }
    }
}
