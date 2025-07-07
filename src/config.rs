use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub distro_image_path: String,
    pub distro_image_size: [i32; 2],
    pub hostname: String,
    pub cpu: String,
    pub memory: String,
    pub startup_disk: String,
    pub graphics: String,
    pub serial_num: String,
    pub overview_margins: [i32; 4],
    pub section_space: i32,
    pub logo_space: i32,
    pub system_info_command: String,
    pub software_update_command: String,
    #[serde(rename = "font-family")]
    pub font_family: Option<String>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            distro_image_path: "tux-logo.png".to_string(),
            distro_image_size: [512, 512],
            hostname: "".to_string(),
            cpu: "".to_string(),
            memory: "".to_string(),
            startup_disk: "".to_string(),
            graphics: "".to_string(),
            serial_num: "".to_string(),
            overview_margins: [60, 60, 60, 60],
            section_space: 20,
            logo_space: 60,
            system_info_command: "".to_string(),
            software_update_command: "".to_string(),
            font_family: None,
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let mut config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        // Check if the image path exists, if not, use tux-logo.png as fallback
        if !std::path::Path::new(&config.distro_image_path).exists() {
            // Try different possible paths for tux-logo.png
            let fallback_paths = vec![
                "tux-logo.png".to_string(),
                "./tux-logo.png".to_string(),
                format!("{}/tux-logo.png", std::env::current_dir().unwrap_or_default().to_string_lossy()),
            ];
            
            let mut found_fallback = false;
            for fallback_path in fallback_paths {
                if std::path::Path::new(&fallback_path).exists() {
                    config.distro_image_path = fallback_path;
                    found_fallback = true;
                    break;
                }
            }
            
            if !found_fallback {
                config.distro_image_path = "tux-logo.png".to_string();
            }
        }
        
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if !std::path::Path::new(&self.distro_image_path).exists() {
            return Err(anyhow::anyhow!(
                "Distro image not found: {}",
                self.distro_image_path
            ));
        }
        
        if self.distro_image_size[0] <= 0 || self.distro_image_size[1] <= 0 {
            return Err(anyhow::anyhow!("Invalid image size"));
        }
        
        if self.overview_margins.len() != 4 {
            return Err(anyhow::anyhow!("Invalid margins array"));
        }
        
        Ok(())
    }
}
