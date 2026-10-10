mod mappings;

use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};

use serde::Deserialize;

use crate::BlockStatePaletteEntry;

pub(crate) static MAPPINGS: LazyLock<UpdateMappings> = LazyLock::new(|| UpdateMappings::get());

pub struct UpdateMappings {
    steps: Vec<Step>,
}

impl UpdateMappings {
    fn get() -> Self {
        Self {
            steps: mappings::mapping_strings()
                .into_iter()
                .map(|(data_version, json)| Step::parse_json(data_version, json))
                .collect(),
        }
    }

    /// Update a palette entry from version `data_version` to the latest this crate supports.
    ///
    /// Block palettes WITH properties searches for a rule in `with_props_rules`. If a rule
    /// is not found, it then searches through `name_only_rules`.
    ///
    /// Block palettes WITHOUT properties only search `name_only_rules`.
    pub(crate) fn update(&self, data_version: i32, palette: &mut BlockStatePaletteEntry) {
        let starting_step_index = self
            .steps
            .partition_point(|step| step.data_version <= data_version);
        for step in &self.steps[starting_step_index..] {
            if !palette.properties.is_empty() {
                if let Some(rule) = step.with_props_rules.get(palette) {
                    rule.apply(palette);
                    continue;
                }
            }
            if let Some(rule) = step.name_only_rules.get(&palette.name) {
                rule.apply(palette);
                continue;
            }
        }
    }
}

/// Contains rules to update incrementally, TO `data_version`.
///
/// `with_props_rules` only match block palettes with the exact property list;
/// `name_only_rules` match all palettes with the same name.
struct Step {
    data_version: i32,
    // stores keys with properties: "chain[waterlogged=true]" -> Rule
    with_props_rules: HashMap<BlockStatePaletteEntry, Rule>,
    // Stores keys without properties: "chain" -> Rule
    name_only_rules: HashMap<String, Rule>,
}

impl Step {
    /// Parses mapping JSON into an update step.
    fn parse_json(data_version: i32, json: &str) -> Step {
        #[derive(Deserialize)]
        struct RawMapping<'a> {
            #[serde(borrow)]
            blockstates: HashMap<&'a str, &'a str>,
        }
        let raw: RawMapping = serde_json::from_str(json).unwrap();

        let mut with_props_rules = HashMap::new();
        let mut name_only_rules = HashMap::new();

        for (key_str, rule_str) in raw.blockstates {
            let rule = if let Some(name) = rule_str.strip_suffix('[') {
                Rule::Rename(format!("minecraft:{name}"))
            } else {
                Rule::Swap(Self::parse_palette(rule_str))
            };

            let key = Self::parse_palette(key_str);
            if key.properties.is_empty() {
                name_only_rules.insert(key.name, rule);
            } else {
                with_props_rules.insert(key, rule);
            }
        }

        Step {
            data_version,
            with_props_rules,
            name_only_rules,
        }
    }

    /// Parses `name` or `name[k=v,...]`, into a palette entry.
    fn parse_palette(state: &str) -> BlockStatePaletteEntry {
        let (name, properties) = match state.strip_suffix(']').and_then(|s| s.split_once('[')) {
            Some((name, props)) => (
                name,
                props
                    .split(',')
                    .filter_map(|prop| prop.split_once('='))
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .collect(),
            ),
            None => (state, BTreeMap::new()),
        };
        BlockStatePaletteEntry {
            name: format!("minecraft:{name}"),
            properties,
        }
    }
}

enum Rule {
    Swap(BlockStatePaletteEntry),
    Rename(String),
}

impl Rule {
    fn apply(&self, palette: &mut BlockStatePaletteEntry) {
        match self {
            Self::Rename(name) => palette.name.clone_from(name),
            Self::Swap(new_palette) => {
                palette.name.clone_from(&new_palette.name);
                palette.properties.clone_from(&new_palette.properties);
            }
        }
    }
}
