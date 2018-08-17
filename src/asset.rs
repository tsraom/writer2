use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Asset {
    path: PathBuf,
    asset_type: AssetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Css,
    Js,
    Other,
}

impl Asset {
    pub fn new(path: PathBuf, asset_type: AssetType) -> Self {
        Self {
            path: path,
            asset_type: asset_type,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn asset_type(&self) -> &AssetType {
        &self.asset_type
    }
}

impl AssetType {
    pub fn guess(path: &PathBuf) -> AssetType {
        let ext = (*path).extension().unwrap();
        if ext == "css" {
            return AssetType::Css;
        }

        if ext == "js" {
            return AssetType::Js;
        }

        AssetType::Other
    }
}
