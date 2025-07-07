use anyhow::Result;
use clap::Parser;
use gtk::prelude::*;
use std::path::PathBuf;

mod config;
mod configurator;
mod main_window;
mod system_info;
mod utils;

use config::Config;
use configurator::ConfiguratorWindow;
use main_window::MainWindow;

const VERSION: &str = "0.9.0";
const AUTHOR: &str = "Kamil 'Novik' Nowicki";
const AUTHOR_EMAIL: &str = "kamil.nowicki@h4b.uk";
const ORIGINAL_AUTHOR: &str = "hungngocphat01";
const ORIGINAL_REPO: &str = "https://github.com/hungngocphat01/AboutThisMc";
const CURRENT_REPO: &str = "https://github.com/n0vik/about-this-linux.git";

#[derive(Parser)]
#[command(author = "Kamil 'Novik' Nowicki <kamil.nowicki@h4b.uk>")]
#[command(version = "0.9.0")]
#[command(about = "A customizable 'About this Mac' dialog for Linux systems")]
struct Cli {
    /// Run the configuration wizard
    #[arg(long)]
    configure: bool,

    /// Path to configuration file
    #[arg(long = "config-path", value_name = "PATH")]
    config_path: Option<String>,

    /// Load a custom overview configuration file
    #[arg(long = "load-overview", value_name = "FILE")]
    load_overview: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let configure = args.configure;
    let config_path = args.config_path;
    let load_overview = args.load_overview;
    
    // Initialize GTK after parsing arguments
    let app = gtk::Application::builder()
        .application_id("com.novik.about-this-linux")
        .build();
    
    // Set application icon
    if let Ok(_) = gdk_pixbuf::Pixbuf::from_file("tux-logo.png") {
        // Icon set successfully
    } else if let Ok(_) = gdk_pixbuf::Pixbuf::from_file("./tux-logo.png") {
        // Icon set successfully
    }

    app.connect_activate(move |app| {
        // Ensure the app follows the system theme
        if let Some(settings) = gtk::Settings::default() {
            // Check if user prefers dark theme
            if is_dark_theme_preferred() {
                settings.set_gtk_application_prefer_dark_theme(true);
            } else {
                settings.set_gtk_application_prefer_dark_theme(false);
            }
        }
        
        if configure {
            let default_config_path = get_default_config_path();
            let config_path = config_path
                .as_ref()
                .map(|p| PathBuf::from(p))
                .unwrap_or_else(|| default_config_path.clone());
            
            let configurator = ConfiguratorWindow::new(app, config_path);
            configurator.present();
        } else if let Some(ref overview_path) = load_overview {
            let config_path = PathBuf::from(overview_path);
            match Config::load(&config_path) {
                Ok(config) => {
                    let main_window = MainWindow::new(app, config);
                    main_window.present();
                }
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            // Auto-detect system information and create config
            let config = create_auto_detected_config();
            let main_window = MainWindow::new(app, config);
            main_window.present();
        }
    });

    app.run_with_args(&["about-this-linux"]);
    Ok(())
}

fn is_dark_theme_preferred() -> bool {
    // Check various sources for dark theme preference
    
    // 1. Check GTK theme name
    if let Some(settings) = gtk::Settings::default() {
        if let Some(theme_name) = settings.gtk_theme_name() {
            let theme_name = theme_name.as_str().to_lowercase();
            if theme_name.contains("dark") || theme_name.contains("adwaita-dark") {
                return true;
            }
        }
    }
    
    // 2. Check gsettings
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output() {
        let theme = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        if theme.contains("dark") {
            return true;
        }
    }
    
    // 3. Check color scheme preference
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
        .output() {
        let scheme = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if scheme.contains("dark") || scheme.contains("prefer-dark") {
            return true;
        }
    }
    
    // 4. Check KDE settings
    if let Ok(output) = std::process::Command::new("kreadconfig5")
        .args(&["--group", "General", "--key", "ColorScheme"])
        .output() {
        let scheme = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        if scheme.contains("dark") {
            return true;
        }
    }
    
    // Default to false (light theme)
    false
}

fn create_auto_detected_config() -> Config {
    // Detect system information
    let system_info = system_info::SystemInfo::detect().unwrap_or_else(|_| {
        system_info::SystemInfo {
            hostname: "Unknown Host".to_string(),
            cpu: "Unknown CPU".to_string(),
            memory: "Unknown Memory".to_string(),
            startup_disk: "Unknown Disk".to_string(),
            graphics: "Unknown Graphics".to_string(),
            serial_number: "Unknown".to_string(),
        }
    });
    
    // Detect distribution-specific logo and size
    let (logo_path, logo_size) = detect_system_logo();
    
    Config {
        distro_image_path: logo_path,
        distro_image_size: logo_size,
        hostname: system_info.hostname,
        cpu: system_info.cpu,
        memory: system_info.memory,
        startup_disk: system_info.startup_disk,
        graphics: system_info.graphics,
        serial_num: system_info.serial_number,
        overview_margins: [60, 60, 60, 60],
        section_space: 20,
        logo_space: 60,
        system_info_command: "".to_string(),
        software_update_command: "".to_string(),
        font_family: None,
    }
}

fn detect_system_logo() -> (String, [i32; 2]) {
    // Try to detect distribution
    let distro_info = system_info::DynamicSystemInfo::detect().unwrap_or_else(|_| {
        system_info::DynamicSystemInfo {
            distro_name: "Unknown Linux".to_string(),
            distro_version: "Unknown".to_string(),
            distro_codename: None,
            kernel: "Unknown".to_string(),
        }
    });
    
    let distro_lower = distro_info.distro_name.to_lowercase();
    
    // Define distribution-specific logos with their preferred sizes
    let logo_configs = vec![
        ("arch", "/usr/share/pixmaps/archlinux-logo.png", [256, 256]),
        ("arch", "/usr/share/icons/hicolor/scalable/apps/archlinux-logo.svg", [256, 256]),
        ("ubuntu", "/usr/share/pixmaps/ubuntu-logo.png", [256, 256]),
        ("ubuntu", "/usr/share/icons/hicolor/scalable/apps/ubuntu-logo.svg", [256, 256]),
        ("fedora", "/usr/share/pixmaps/fedora-logo.png", [256, 256]),
        ("fedora", "/usr/share/icons/hicolor/scalable/apps/fedora-logo.svg", [256, 256]),
        ("debian", "/usr/share/pixmaps/debian-logo.png", [256, 256]),
        ("debian", "/usr/share/icons/debian-logo.png", [256, 256]),
        ("opensuse", "/usr/share/pixmaps/opensuse-logo.png", [256, 256]),
        ("suse", "/usr/share/pixmaps/opensuse-logo.png", [256, 256]),
        ("manjaro", "/usr/share/pixmaps/manjaro.png", [256, 256]),
        ("manjaro", "/usr/share/icons/hicolor/scalable/apps/manjaro.svg", [256, 256]),
        ("mint", "/usr/share/pixmaps/linuxmint-logo.png", [256, 256]),
        ("elementary", "/usr/share/pixmaps/distributor-logo.png", [256, 256]),
        ("pop", "/usr/share/pixmaps/pop-logo.png", [256, 256]),
        ("zorin", "/usr/share/pixmaps/zorin-logo.png", [256, 256]),
        ("kali", "/usr/share/pixmaps/kali-dragon-logo.png", [256, 256]),
        ("centos", "/usr/share/pixmaps/centos-logo.png", [256, 256]),
        ("rhel", "/usr/share/pixmaps/redhat-logo.png", [256, 256]),
        ("redhat", "/usr/share/pixmaps/redhat-logo.png", [256, 256]),
    ];
    
    // Try to find distribution-specific logo
    for (distro_name, logo_path, size) in logo_configs {
        if distro_lower.contains(distro_name) && std::path::Path::new(logo_path).exists() {
            return (logo_path.to_string(), size);
        }
    }
    
    // Try some generic locations
    let generic_paths = vec![
        "/usr/share/pixmaps/distributor-logo.png",
        "/usr/share/icons/hicolor/scalable/apps/distributor-logo.svg",
        "/usr/share/pixmaps/linux-logo.png",
        "/usr/share/icons/hicolor/48x48/apps/distributor-logo.png",
        "/usr/share/icons/hicolor/64x64/apps/distributor-logo.png",
        "/usr/share/icons/hicolor/128x128/apps/distributor-logo.png",
    ];
    
    for path in generic_paths {
        if std::path::Path::new(path).exists() {
            return (path.to_string(), [256, 256]);
        }
    }
    
    // Check for desktop environment specific logos
    let de_logos = vec![
        "/usr/share/pixmaps/gnome-logo.png",
        "/usr/share/pixmaps/kde-logo.png",
        "/usr/share/pixmaps/xfce-logo.png",
    ];
    
    for path in de_logos {
        if std::path::Path::new(path).exists() {
            return (path.to_string(), [256, 256]);
        }
    }
    
    // Fallback to tux logo
    let tux_paths = vec![
        "tux-logo.png",
        "./tux-logo.png",
        "/usr/share/pixmaps/tux.png",
    ];
    
    for path in tux_paths {
        if std::path::Path::new(path).exists() {
            return (path.to_string(), [256, 256]);
        }
    }
    
    // Ultimate fallback
    ("tux-logo.png".to_string(), [256, 256])
}

fn get_default_config_path() -> PathBuf {
    let mut config_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    config_dir.push(".local/share/about-this-linux");
    std::fs::create_dir_all(&config_dir).unwrap_or_default();
    config_dir.push("overview-conf.json");
    config_dir
}
