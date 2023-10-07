//! Bevy Commands that can be run to update the game state.
//! Used to process Yarn Spinner commands using pre-registered
//! [CommandHandlerFn] command handlers.

use std::collections::HashMap;

use bevy::{
    ecs::system::Command,
    prelude::{warn, App, Mut, Resource, World},
};

/// Represents a "command handler", which is a way for Bevy apps to register
/// functions that are called in response to commands parsed from the Yarn file.
/// For instance if the yarn file has `<<my_command abcd>>`, then a command handler
/// with the name `my_command` can be reigstered and the associated handler function
/// will be called when this is found in the Yarn file.
///
/// Any commands that do not have an associated handler will use a [events::BevyYarnEvent]
/// to raise that with the bevy application.
pub type CommandHandlerFn = fn(&mut World, Vec<String>);

#[derive(Default, Resource)]
pub(crate) struct CommandHandlers(pub(crate) HashMap<String, CommandHandlerFn>);

/// Represents a custom command from within the Yarn file, usually expressed as
///
/// ```yarn
/// <<my_command arg1 arg2 argN>>
/// ```
#[derive(Debug, Clone)]
pub struct BevyYarnCommand {
    /// The name of the command
    pub command_name: String,

    /// The arguments provided to the command
    pub args: Vec<String>,

    /// Whether the command has already been handled by a pre-registered command
    pub handled: bool,
}

impl Command for BevyYarnCommand {
    // This approach is inspired by https://github.com/Semihazah/bevy_yarn_spinner
    fn apply(self, world: &mut World) {
        world.resource_scope(|world, command_registry: Mut<CommandHandlers>| {
            if let Some(handler) = command_registry.0.get(&self.command_name) {
                handler(world, self.args);
            }
        });
    }
}

/// A trait that allows registering or replacing [CommandHandlerFn] handlers after
/// the [crate::YarnPlugin] has been added to the bevy App.
pub trait AddBevyCommandHandlerExt {
    /// Add a command to the [CommandHandlers] for this app. If the command already exists,
    /// the existing handler is replaced.
    fn add_yarn_command<N: Into<String>>(
        &mut self,
        command_name: N,
        handler: CommandHandlerFn,
    ) -> &mut Self;
}

impl AddBevyCommandHandlerExt for World {
    fn add_yarn_command<N: Into<String>>(
        &mut self,
        command_name: N,
        handler: CommandHandlerFn,
    ) -> &mut Self {
        match self.get_resource_mut::<CommandHandlers>() {
            Some(mut handlers) => {
                handlers.0.insert(command_name.into(), handler);
            },
            None => warn!("Attempted to add YarnCommand, but no CommandHandlers present. Was the YarnPlugin added?"),
        };

        self
    }
}

impl AddBevyCommandHandlerExt for App {
    fn add_yarn_command<N: Into<String>>(
        &mut self,
        command_name: N,
        handler: CommandHandlerFn,
    ) -> &mut Self {
        let _ = self.world.add_yarn_command(command_name, handler);
        self
    }
}
