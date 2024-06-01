use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct AssetTracker {
    bundles: HashMap<String, Vec<PathBuf>>,
}

impl AssetTracker {
    pub fn search_bundle_files(&mut self, bundles_path: &Path) -> anyhow::Result<()> {
        for file in WalkDir::new(bundles_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|f| f.path().is_file())
        {
            let file_path = file.path().canonicalize()?;
            let stripped_file_path = file_path.strip_prefix(bundles_path)?;
            let mut file_parts = stripped_file_path.iter();

            let bundle_name = file_parts
                .next()
                .unwrap()
                .to_string_lossy()
                .to_mut()
                .clone();
            let asset_path = file_parts.collect::<PathBuf>();

            if let Some(file_list) = self.bundles.get_mut(&bundle_name) {
                file_list.push(asset_path);
            } else {
                self.bundles.insert(bundle_name, vec![asset_path]);
            }
        }

        Ok(())
    }

    pub fn condence(&self, bundle_order: &Vec<String>) -> anyhow::Result<Vec<PathBuf>> {
        let mut map = HashMap::new();

        for bundle in bundle_order {
            let paths = self
                .bundles
                .get(bundle)
                .with_context(|| format!("Failed to find tracked bundle: {}", bundle))?;

            for path in paths {
                map.insert(path, bundle.clone());
            }
        }

        Ok(map
            .iter()
            .map(|(path, bundle)| PathBuf::from(bundle).join(path))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn condence() {
        let order = vec!["bundle1".to_string(), "bundle2".to_string()];

        let asset_tracker = AssetTracker {
            bundles: HashMap::from([
                (
                    "bundle1".to_string(),
                    vec![
                        PathBuf::from("minecraft/blockstates/dirt.json"),
                        PathBuf::from("minecraft/models/block/grass_block.json"),
                    ],
                ),
                (
                    "bundle2".to_string(),
                    vec![
                        PathBuf::from("minecraft/blockstates/dirt.json"),
                        PathBuf::from("minecraft/models/block/dirt.json"),
                    ],
                ),
            ]),
        };

        let expected = HashSet::from([
            PathBuf::from("bundle2/minecraft/blockstates/dirt.json"),
            PathBuf::from("bundle2/minecraft/models/block/dirt.json"),
            PathBuf::from("bundle1/minecraft/models/block/grass_block.json"),
        ]);

        let condenced = asset_tracker.condence(&order).unwrap();

        assert_eq!(HashSet::from_iter(condenced), expected);
    }
}
