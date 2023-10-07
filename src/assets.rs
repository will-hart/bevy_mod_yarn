//! Custom asset loaders for compiled Yarn files (yarnc) and
//! the associated string and metadata files as defined in [crate::data].

use std::{collections::HashMap, path::PathBuf};

use bevy::{
    asset::{AssetLoader, AssetPath, LoadedAsset},
    prelude::{warn, Handle},
    reflect::{TypePath, TypeUuid},
};
use csv::{Reader, ReaderBuilder};
use prost::Message;
use regex::Regex;
use yharnam::{expand_format_functions, Line, LineInfo, MetadataInfo, Program};

/// A newtype wrapping a yarn spinner program that can be loaded
/// into the bevy engine.
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "e0021061-5ff9-4134-992e-a9352d8854cd"]
pub struct BevyYarnProgram {
    /// The program loaded from the yarnc file
    pub program: Program,

    /// A handle for the string table for this yarnc file
    pub string_table: Handle<BevyYarnStringTable>,

    /// A handle for the metadata table for this yarnc file
    pub metadata_table: Handle<BevyYarnMetadataTable>,
}

pub(crate) fn get_table_pathbuf_from_yarnc_path<P>(yarnc_path: P, prefix: &str) -> PathBuf
where
    P: Into<PathBuf>,
{
    let mut pb: PathBuf = yarnc_path.into();
    pb.set_file_name(format!(
        "{}.{prefix}.csv",
        pb.file_stem().unwrap().to_str().unwrap()
    ));
    pb
}

/// A custom loader for BevyYarnProgram assets.
#[derive(Default)]
pub struct BevyYarnProjectAssetLoader;

impl AssetLoader for BevyYarnProjectAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            // First load in the program from the yarnc file
            let program = Program::decode(bytes)?;

            // Next load the string table, it should have the name `<yarnc-file-name>-Lines.csv`
            let path = get_table_pathbuf_from_yarnc_path(load_context.path(), "lines");
            let string_asset_path = AssetPath::new(path, None);
            let string_table: Handle<BevyYarnStringTable> =
                load_context.get_handle(string_asset_path.clone());

            // Next load the metadata table, it should have the name `<yarnc-file-name>-Metadata.csv`
            let path = get_table_pathbuf_from_yarnc_path(load_context.path(), "metadata");
            let metadata_asset_path = AssetPath::new(path, None);
            let metadata_table: Handle<BevyYarnMetadataTable> =
                load_context.get_handle(metadata_asset_path.clone());

            // Finally set all the loaded assets and mark the tables as dependencies
            load_context.set_default_asset(
                LoadedAsset::new(BevyYarnProgram {
                    program,
                    string_table,
                    metadata_table,
                })
                .with_dependencies(vec![string_asset_path, metadata_asset_path]),
            );

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["yarnc"]
    }
}

/// A resource to contain the string table
#[derive(Default, Debug, TypeUuid, TypePath)]
#[uuid = "d11069b5-98c8-4db0-8616-58d86ee1deb3"]
pub struct BevyYarnStringTable(pub HashMap<String, LineInfo>);

impl BevyYarnStringTable {
    /// Finds the string for a line from the given string table
    fn find_string_in_table(&self, id: &String) -> String {
        if let Some(text) = self.0.get(id).map(|line_info| line_info.text.clone()) {
            text
        } else {
            warn!("Line id {id} missing from string table. Skipping");
            format!("<missing_string: {id}>")
        }
    }

    /// Completes variable substitutions in the given string
    fn perform_variable_substitutions(initial: String, substitutions: &[String]) -> String {
        substitutions
            .iter()
            .enumerate()
            .fold(initial, |current, (idx, next_sub)| {
                current.replace(&format!("{{{idx}}}"), next_sub)
            })
    }

    /// Pulls out the character (if any) from the given formatted string.
    /// Characters are represented by e.g. "character 1 name: line" in the yarn file
    fn extract_character(formatted_text: String) -> (Option<String>, String) {
        let character_regex: Regex = Regex::new(r"([a-zA-Z0-9]+:)?\s*(.*)").unwrap();

        match character_regex.captures(&formatted_text) {
            Some(captures) => {
                if captures.len() == 3 {
                    (
                        captures
                            .get(1)
                            .map(|val| val.as_str().to_owned().replace(':', "")),
                        captures.get(2).unwrap().as_str().to_owned(),
                    )
                } else {
                    (None, formatted_text)
                }
            }
            None => (None, formatted_text),
        }
    }

    /// Gets the final substituted and formatted text
    pub fn get_final_text(&self, line: &Line, local_code: &str) -> (Option<String>, String) {
        let initial = self.find_string_in_table(&line.id);
        let (character, initial) = Self::extract_character(initial);
        let subbed_text = Self::perform_variable_substitutions(initial, &line.substitutions);
        (character, expand_format_functions(&subbed_text, local_code))
    }
}

/// A custom loader for BevyYarnProgram assets.
#[derive(Default)]
pub struct BevyYarnStringTableAssetLoader;

impl AssetLoader for BevyYarnStringTableAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let string_table =
                HashMap::from_iter(Reader::from_reader(bytes).deserialize().map(|result| {
                    let res: LineInfo = result.unwrap();
                    (res.id.clone(), res)
                }));

            load_context.set_default_asset(LoadedAsset::new(BevyYarnStringTable(string_table)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lines.csv"]
    }
}

/// A resource to contain the metadata table
#[derive(Default, Debug, TypeUuid, TypePath)]
#[uuid = "42073437-7c2b-4526-859c-f1b059881c67"]
pub struct BevyYarnMetadataTable(pub HashMap<String, MetadataInfo>);

impl BevyYarnMetadataTable {
    /// Gets the tags associated with a given line, if any
    pub fn get_tags_for_line(&self, line: &Line) -> Vec<String> {
        self.0
            .get(&line.id)
            .map(|metadata_info| &metadata_info.tags)
            .cloned()
            .unwrap_or_default()
    }
}

/// A custom loader for BevyYarnProgram assets.
#[derive(Default)]
pub struct BevyYarnMetadataTableAssetLoader;

impl AssetLoader for BevyYarnMetadataTableAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let metadata_table = HashMap::from_iter(
                ReaderBuilder::new()
                    .flexible(true)
                    .from_reader(bytes)
                    .deserialize()
                    .map(|result| {
                        if result.is_err() {
                            warn!("[{:?}] {result:?}\n", load_context.path());
                        }

                        let res: MetadataInfo = result.unwrap();
                        (res.id.clone(), res)
                    }),
            );

            load_context.set_default_asset(LoadedAsset::new(BevyYarnMetadataTable(metadata_table)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["metadata.csv"]
    }
}
