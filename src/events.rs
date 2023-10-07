//! Events that are used to inject data from the Yarn state machine
//! into the bevy ECS.

use bevy::prelude::Event;

use crate::prelude::{BevyYarnChoice, BevyYarnCommand, BevyYarnLine};

/// An event that is raised when the dialogue should step forward
#[derive(Event)]
pub struct BevyYarnStepDialogueEvent;

/// Events that can be raised by the YarnEngine for processing
/// within bevy (usually by client code)
#[derive(Clone, Debug, Event)]
pub enum BevyYarnEvent {
    /// Say a line
    Say(BevyYarnLine),
    /// Offer some choices
    Choices(Vec<BevyYarnChoice>),
    /// Run a command
    Command(BevyYarnCommand),
    /// End the conversation
    EndConversation,
}
