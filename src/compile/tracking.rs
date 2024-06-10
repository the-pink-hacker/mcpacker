use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct AssetTracker {
    bundles: HashMap<PathBuf, Vec<PathBuf>>,
}

impl AssetTracker {
    pub fn search_bundle_files(&mut self, bundles_path: &Path) -> anyhow::Result<()> {
        for file in WalkDir::new(bundles_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|f| f.path().is_file())
        {
            let file_path = file.path().canonicalize()?;
            let mut stripped_path = file_path.strip_prefix(bundles_path)?.iter();

            let bundle_name = stripped_path.next().with_context(|| {
                format!(
                    "Failed to get bundle name from asset path: {}",
                    file_path.display()
                )
            })?;
            let bundle_path = bundles_path.join(bundle_name);

            let asset_path = stripped_path.collect::<PathBuf>();

            if let Some(file_list) = self.bundles.get_mut(&bundle_path) {
                file_list.push(asset_path);
            } else {
                self.bundles.insert(bundle_path, vec![asset_path]);
            }
        }

        Ok(())
    }

    pub fn condence(&self, bundle_order: &Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
        let mut map = HashMap::new();

        for bundle in bundle_order {
            let paths = self
                .bundles
                .get(bundle)
                .with_context(|| format!("Failed to find tracked bundle: {}", bundle.display()))?;

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
        let order = vec!["bundle1".into(), "bundle2".into()];

        let asset_tracker = AssetTracker {
            bundles: HashMap::from([
                (
                    "bundle1".into(),
                    vec![
                        "minecraft/blockstates/dirt.json".into(),
                        "minecraft/models/block/grass_block.json".into(),
                    ],
                ),
                (
                    "bundle2".into(),
                    vec![
                        "minecraft/blockstates/dirt.json".into(),
                        "minecraft/models/block/dirt.json".into(),
                    ],
                ),
            ]),
        };

        let expected = HashSet::from([
            "bundle2/minecraft/blockstates/dirt.json".into(),
            "bundle2/minecraft/models/block/dirt.json".into(),
            "bundle1/minecraft/models/block/grass_block.json".into(),
        ]);

        let condenced = asset_tracker.condence(&order).unwrap();

        assert_eq!(HashSet::from_iter(condenced), expected);
    }
}
