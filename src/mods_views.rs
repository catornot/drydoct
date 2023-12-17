use anyhow::{Ok, Result as AnyResult};
use ratatui::{prelude::*, text::Line};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thermite::{
    model::{Manifest, ModJSON},
    prelude::ThermiteError,
};

#[derive(Debug)]
pub enum ModView {
    Mods(Vec<InstalledModPartial>),
    Plugins(Vec<InstalledPlugins>),
    Packages(Vec<()>),
    CTA,
}

impl Default for ModView {
    fn default() -> Self {
        Self::Mods(Vec::default())
    }
}

impl ModView {
    pub fn reload(&mut self, game_path: &Path) -> AnyResult<()> {
        match self {
            ModView::Mods(mods) => *mods = reload_mods(game_path)?,
            ModView::Plugins(plugins) => *plugins = reload_plugins(game_path)?,
            ModView::Packages(_) => {}
            ModView::CTA => {}
        };

        Ok(())
    }

    pub fn filter(&mut self, keyword: &str) {
        match self {
            ModView::Mods(mods) => {
                *mods = mods
                    .clone()
                    .into_iter()
                    .filter(|nsmod| nsmod.mod_json.name.find(keyword).is_some())
                    .collect();
            }
            ModView::Plugins(plugins) => {
                *plugins = plugins
                    .clone()
                    .into_iter()
                    .filter(|plugin| plugin.name.find(keyword).is_some())
                    .collect();
            }
            ModView::Packages(_) => {}
            ModView::CTA => {}
        };
    }

    pub fn switch(&mut self, view_index: usize) -> AnyResult<&mut Self> {
        *self = match view_index {
            0 => ModView::Mods(Vec::new()),
            1 => ModView::Plugins(Vec::new()),
            2 => ModView::Packages(Vec::new()),
            3 => ModView::CTA,
            _ => {
                log::warn!("somehow got invalid view index of {}", view_index);
                ModView::Mods(Vec::new())
            }
        };

        Ok(self)
    }

    pub fn get_as_paragraph(&self, index: usize) -> Option<Text<'_>> {
        match self {
            ModView::Mods(mods) => {
                let nsmod = mods.get(index)?;
                Some(Text::from(vec![
                    Line::from(vec![
                        Span::raw(&nsmod.mod_json.version),
                        Span::raw("|"),
                        Span::styled(
                            nsmod
                                .mod_json
                                .load_priority
                                .unwrap_or_else(|| 999)
                                .to_string(),
                            Style::default().green(),
                        ),
                    ]),
                    Line::raw(&nsmod.mod_json.description),
                ]))
            }
            ModView::Plugins(plugins) => plugins.get(index).map(|plugin| Text::raw(&plugin.name)),
            ModView::Packages(_) => None,
            ModView::CTA => None,
        }
    }

    pub fn get_title(&self, index: usize) -> Option<&str> {
        match self {
            ModView::Mods(mods) => mods.get(index).map(|nsmod| nsmod.mod_json.name.as_str()),
            ModView::Plugins(plugins) => plugins.get(index).map(|plugin| plugin.name.as_str()),
            ModView::Packages(_) => Some(""),
            ModView::CTA => Some("CTA VIEW"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ModView::Mods(mods) => mods.len(),
            ModView::Plugins(plugins) => plugins.len(),
            ModView::Packages(_) => 0,
            ModView::CTA => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstalledModPartial {
    pub manifest: Option<Manifest>,
    pub mod_json: ModJSON,
    pub author: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InstalledPlugins {
    name: String, // too lazy to fetch everthing else XD
}

pub fn reload_plugins(game_path: &Path) -> AnyResult<Vec<InstalledPlugins>> {
    Ok(game_path
        .join("R2Northstar")
        .join("plugins")
        .as_path()
        .canonicalize()?
        .read_dir()?
        .filter_map(|file| file.ok())
        .filter(|file| file.file_type().map(|ty| ty.is_file()).unwrap_or_default())
        .map(|file| file.path())
        .filter(|path| {
            path.extension()
                .map(|ext| ext.eq("dll"))
                .unwrap_or_default()
        })
        .filter_map(|file| Some(file.file_name()?.to_str()?.to_owned()))
        .map(|name| InstalledPlugins { name })
        .collect())
}
pub fn reload_mods(game_path: &Path) -> AnyResult<Vec<InstalledModPartial>> {
    Ok(
        find_mods(game_path.join("R2Northstar").join("mods").as_path())?
            .into_iter()
            .filter_map(|nsmod| nsmod.ok())
            .collect(),
    )
    // Ok(find_mods(&mods_path)?
    //     .into_iter()
    //     .collect::<Result<Vec<InstalledModPartial>, ThermiteError>>()?)
}

fn find_mods(dir: &Path) -> Result<Vec<Result<InstalledModPartial, ThermiteError>>, ThermiteError> {
    Result::Ok(
        dir.canonicalize()?
            .read_dir()?
            .filter_map(|file| file.ok())
            .filter(|file| file.file_type().map(|ty| ty.is_dir()).unwrap_or_default())
            .map(|dir| dir.path())
            .map(extract_mod_info)
            .collect(),
    )
}

fn extract_mod_info(mod_dir: PathBuf) -> Result<InstalledModPartial, ThermiteError> {
    Result::Ok(InstalledModPartial {
        manifest: fs::read_to_string(mod_dir.join("manifest.json"))
            .ok()
            .map(|manifest| json5::from_str(&manifest).ok())
            .flatten(),
        mod_json: json5::from_str(fs::read_to_string(mod_dir.join("mod.json"))?.as_str())?,
        author: fs::read_to_string(mod_dir.join("thunderstore_author.txt")).ok(),
        path: mod_dir,
    })
}
