//! A component representing a new yarn data file to load into the engine

use bevy::prelude::Component;
use chapter::Line;

/// A component that is added to trigger loading a yarn engine.  The entity that this component
/// is added has the yharnam "Virtual Machine" added to it and this component is removed.
///
/// The string CSV file and metadata CSV file are automatically loaded when the yarnc program
/// is loaded. These files must be located at the same place as the yarnc file, for instance
/// the following three files should be present in the same directory:
///
/// 1. mystory.yarnc
/// 2. mystory-Lines.csv
/// 3. mystory-Metadata.csv
///
#[derive(Component)]
pub struct YarnData {
    /// The path to load the yarnc file from from
    pub yarnc_path: String,
}

/// Represents a choice that can be made, including some metadata
#[derive(Debug, Clone)]
pub struct BevyYarnChoice {
    /// The line ID for this choice
    pub line_id: String,
    /// The destination node that this choice navigates to
    pub destination_node: String,
    /// The line to display for this choice
    pub formatted_line: BevyYarnLine,
}

/// Represents a line that that can be said, including some metadata
#[derive(Debug, Clone)]
pub struct BevyYarnLine {
    /// The line metadata from the Yarn engine
    pub line: Line,
    /// The formatted text, including any substitutions and with formatting functions expanded
    pub formatted_text: String,
    /// If the line is prefixed with "<characer name>: ", this is trimmed from the text and available here.
    pub character: Option<String>,
    /// A list of tags associated with this line
    pub tags: Vec<String>,
}
