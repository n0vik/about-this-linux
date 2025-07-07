use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub cpu: String,
    pub memory: String,
    pub startup_disk: String,
    pub graphics: String,
    pub serial_number: String,
}

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub displays: Vec<Display>,
}

#[derive(Debug, Clone)]
pub struct Display {
    pub name: String,
    pub resolution: String,
    pub refresh_rate: String,
    pub color_depth: String,
    pub is_primary: bool,
    pub brightness: String,
    pub rotation: String,
    pub scale_factor: String,
    pub color_profile: String,
    pub connection_type: String,
}

#[derive(Debug, Clone)]
pub struct DynamicSystemInfo {
    pub distro_name: String,
    pub distro_version: String,
    pub distro_codename: Option<String>,
    pub kernel: String,
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub devices: Vec<StorageDevice>,
    pub filesystems: Vec<Filesystem>,
}

#[derive(Debug, Clone)]
pub struct StorageDevice {
    pub name: String,
    pub model: String,
    pub size: String,
    pub device_type: String, // SSD, HDD, NVMe, etc.
    pub interface: String,   // SATA, NVMe, USB, etc.
    pub serial: String,
    pub temperature: Option<String>,
    pub health: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Filesystem {
    pub device: String,
    pub mountpoint: String,
    pub filesystem_type: String,
    pub total_size: String,
    pub used_size: String,
    pub available_size: String,
    pub usage_percent: f32,
}

impl SystemInfo {
    pub fn detect() -> Result<Self> {
        let fastfetch_info = get_fastfetch_info()?;
        let memory_info = get_memory_info()?;
        let startup_disk = get_startup_disk()?;
        let serial_number = get_serial_number().unwrap_or_else(|_| "Unknown".to_string());

        // Parse hostname
        let hostname = fastfetch_info
            .get("Host")
            .cloned()
            .unwrap_or_else(|| {
                std::env::var("HOSTNAME")
                    .or_else(|_| {
                        Command::new("hostname")
                            .output()
                            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                    })
                    .unwrap_or_else(|_| "Unknown Host".to_string())
            });

        // Parse CPU
        let cpu = fastfetch_info
            .get("CPU")
            .map(|cpu_info| {
                // Reverse the order if it contains @ (frequency)
                if cpu_info.contains('@') {
                    let parts: Vec<&str> = cpu_info.split('@').collect();
                    if parts.len() == 2 {
                        format!("{} {}", parts[1].trim(), parts[0].trim())
                    } else {
                        cpu_info.clone()
                    }
                } else {
                    cpu_info.clone()
                }
            })
            .unwrap_or_else(|| "Unknown CPU".to_string());

        // Parse graphics
        let graphics = fastfetch_info
            .get("GPU")
            .cloned()
            .unwrap_or_else(|| "Unknown Graphics".to_string());

        Ok(SystemInfo {
            hostname,
            cpu,
            memory: memory_info,
            startup_disk,
            graphics,
            serial_number,
        })
    }

    pub fn to_config(&self, image_path: String) -> Config {
        Config {
            distro_image_path: image_path,
            distro_image_size: [512, 512],
            hostname: self.hostname.clone(),
            cpu: self.cpu.clone(),
            memory: self.memory.clone(),
            startup_disk: self.startup_disk.clone(),
            graphics: self.graphics.clone(),
            serial_num: self.serial_number.clone(),
            overview_margins: [60, 60, 60, 60],
            section_space: 20,
            logo_space: 60,
            system_info_command: "".to_string(),
            software_update_command: "".to_string(),
            font_family: None,
        }
    }
}

impl DynamicSystemInfo {
    pub fn detect() -> Result<Self> {
        let os_release_info = get_os_release_info()?;
        let kernel = get_kernel_version()?;

        let distro_name = os_release_info
            .get("NAME")
            .cloned()
            .unwrap_or_else(|| "Unknown Linux".to_string());
        
        let distro_version = os_release_info
            .get("VERSION")
            .cloned()
            .unwrap_or_else(|| "Unknown Version".to_string());

        let distro_codename = os_release_info.get("VERSION_CODENAME").cloned();

        Ok(DynamicSystemInfo {
            distro_name,
            distro_version,
            distro_codename,
            kernel,
        })
    }

    pub fn get_distro_markup(&self) -> String {
        if let Some(ref codename) = self.distro_codename {
            format!(
                "<span font-size='xx-large'><span font-weight='bold'>{} </span>{}</span>",
                self.distro_name, codename
            )
        } else {
            format!(
                "<span font-size='xx-large'><span font-weight='bold'>{}</span></span>",
                self.distro_name
            )
        }
    }
}

fn get_fastfetch_info() -> Result<HashMap<String, String>> {
    let output = Command::new("fastfetch")
        .args(&["--format", "json"])
        .output()
        .or_else(|_| {
            // Fallback to plain text format if JSON fails
            Command::new("fastfetch")
                .args(&["--logo", "none"])
                .output()
        })
        .context("Failed to run fastfetch. Please make sure fastfetch is installed.")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Try to parse as JSON first
    if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&output_str) {
        return parse_fastfetch_json(json_data);
    }
    
    // Fall back to plain text parsing
    parse_fastfetch_text(&output_str)
}

fn parse_fastfetch_json(json_data: serde_json::Value) -> Result<HashMap<String, String>> {
    let mut info = HashMap::new();
    
    if let Some(array) = json_data.as_array() {
        for item in array {
            if let Some(item_type) = item["type"].as_str() {
                match item_type {
                    "Title" => {
                        if let Some(result) = item["result"].as_object() {
                            if let Some(hostname) = result["hostName"].as_str() {
                                info.insert("Host".to_string(), hostname.to_string());
                            }
                            if let Some(username) = result["userName"].as_str() {
                                info.insert("User".to_string(), username.to_string());
                            }
                        }
                    },
                    "CPU" => {
                        if let Some(result) = item["result"].as_object() {
                            if let Some(cpu_name) = result["cpu"].as_str() {
                                let mut cpu_info = cpu_name.to_string();
                                
                                // Add core information if available
                                if let Some(cores) = result["cores"].as_object() {
                                    if let (Some(physical), Some(logical)) = (
                                        cores["physical"].as_u64(),
                                        cores["logical"].as_u64()
                                    ) {
                                        cpu_info.push_str(&format!(" ({} cores, {} threads)", physical, logical));
                                    }
                                }
                                
                                // Add frequency information if available
                                if let Some(freq) = result["frequency"].as_object() {
                                    if let (Some(base), Some(max)) = (
                                        freq["base"].as_u64(),
                                        freq["max"].as_u64()
                                    ) {
                                        cpu_info.push_str(&format!(" @ {:.1}-{:.1} GHz", base as f64 / 1000.0, max as f64 / 1000.0));
                                    }
                                }
                                
                                info.insert("CPU".to_string(), cpu_info);
                            }
                        }
                    },
                    "Memory" => {
                        if let Some(result) = item["result"].as_object() {
                            if let Some(total) = result["total"].as_u64() {
                                let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
                                info.insert("Memory".to_string(), format!("{:.1} GB", total_gb));
                            }
                        }
                    },
                    "GPU" => {
                        if let Some(result) = item["result"].as_array() {
                            if let Some(gpu) = result.get(0).and_then(|g| g.as_object()) {
                                if let Some(gpu_name) = gpu["name"].as_str() {
                                    let mut gpu_info = gpu_name.to_string();
                                    if let Some(vendor) = gpu["vendor"].as_str() {
                                        gpu_info = format!("{} {}", vendor, gpu_info);
                                    }
                                    info.insert("GPU".to_string(), gpu_info);
                                }
                            }
                        }
                    },
                    "Disk" => {
                        if let Some(result) = item["result"].as_array() {
                            // Find the root disk
                            for disk in result {
                                if let Some(disk_obj) = disk.as_object() {
                                    if let Some(mountpoint) = disk_obj["mountpoint"].as_str() {
                                        if mountpoint == "/" {
                                            if let Some(bytes) = disk_obj["bytes"].as_object() {
                                                if let Some(total) = bytes["total"].as_u64() {
                                                    let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
                                                    let filesystem = disk_obj["filesystem"].as_str().unwrap_or("Unknown");
                                                    info.insert("Startup Disk".to_string(), format!("{:.1} GB ({})", total_gb, filesystem));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "Host" => {
                        if let Some(result) = item["result"].as_object() {
                            let mut host_info = String::new();
                            
                            if let Some(vendor) = result["vendor"].as_str() {
                                host_info.push_str(vendor);
                            }
                            
                            if let Some(version) = result["version"].as_str() {
                                if !host_info.is_empty() {
                                    host_info.push(' ');
                                }
                                host_info.push_str(version);
                            }
                            
                            if !host_info.is_empty() {
                                info.insert("Hardware".to_string(), host_info);
                            }
                            
                            if let Some(serial) = result["serial"].as_str() {
                                if !serial.is_empty() {
                                    info.insert("Serial Number".to_string(), serial.to_string());
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    Ok(info)
}

fn parse_fastfetch_text(output_str: &str) -> Result<HashMap<String, String>> {
    // Remove ANSI escape codes
    let ansi_escape = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")?;
    let clean_output = ansi_escape.replace_all(output_str, "");

    let mut info = HashMap::new();
    
    for line in clean_output.lines() {
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                info.insert(key, value);
            }
        }
    }

    Ok(info)
}

fn get_os_release_info() -> Result<HashMap<String, String>> {
    let output = std::fs::read_to_string("/etc/os-release")
        .context("Failed to read /etc/os-release")?;

    let mut info = HashMap::new();
    
    for line in output.lines() {
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..]
                .trim()
                .trim_matches('"')
                .to_string();
            info.insert(key, value);
        }
    }

    Ok(info)
}

fn get_memory_info() -> Result<String> {
    // Try multiple methods to get memory information without root access
    
    // Method 1: Try /proc/meminfo for total memory
    if let Ok(meminfo_content) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0;
        
        for line in meminfo_content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value_part) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = value_part.parse::<u64>() {
                        total_kb = kb;
                        break;
                    }
                }
            }
        }
        
        if total_kb > 0 {
            let total_gb = total_kb as f64 / 1024.0 / 1024.0;
            
            // Try to get additional info from dmidecode without sudo (if available)
            let mut memory_type = String::new();
            let mut speed = String::new();
            
            // Try dmidecode without sudo first
            if let Ok(output) = Command::new("dmidecode")
                .args(&["--type", "memory"])
                .output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                for line in output_str.lines() {
                    let line = line.trim();
                    if line.starts_with("Speed:") && speed.is_empty() {
                        if let Some(speed_part) = line.split(':').nth(1) {
                            speed = speed_part.trim().replace("MT/s", "MHz");
                        }
                    } else if line.starts_with("Type:") && memory_type.is_empty() {
                        if let Some(type_part) = line.split(':').nth(1) {
                            let mem_type = type_part.trim();
                            if !mem_type.is_empty() && mem_type != "Unknown" {
                                memory_type = mem_type.to_string();
                            }
                        }
                    }
                }
            }
            
            // Try lshw as alternative (may work without sudo on some systems)
            if memory_type.is_empty() || speed.is_empty() {
                if let Ok(output) = Command::new("lshw")
                    .args(&["-class", "memory", "-short"])
                    .output() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Parse lshw output for memory type if needed
                    if memory_type.is_empty() {
                        for line in output_str.lines() {
                            if line.contains("memory") && (line.contains("DDR") || line.contains("SDRAM")) {
                                if let Some(ddr_part) = line.split_whitespace().find(|s| s.contains("DDR")) {
                                    memory_type = ddr_part.to_string();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            // Format the result
            let (size, unit) = if total_gb >= 1024.0 {
                (total_gb / 1024.0, "TB")
            } else {
                (total_gb, "GB")
            };
            
            let mut result = format!("{:.1} {}", size, unit);
            
            if !speed.is_empty() {
                result.push_str(&format!(" {}", speed));
            }
            
            if !memory_type.is_empty() {
                result.push_str(&format!(" {}", memory_type));
            }
            
            return Ok(result);
        }
    }
    
    // Method 2: Try using free command
    if let Ok(output) = Command::new("free")
        .args(&["-h"])
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    return Ok(format!("{}B RAM", parts[1]));
                }
            }
        }
    }
    
    // Fallback
    Ok("Unknown Memory".to_string())
}

fn get_startup_disk() -> Result<String> {
    let output = Command::new("lsblk")
        .args(&["-o", "mountpoint,name,label", "--list"])
        .output()
        .context("Failed to run lsblk")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() && parts[0] == "/" {
            if parts.len() >= 3 {
                return Ok(parts[2..].join(" "));
            } else if parts.len() >= 2 {
                return Ok(parts[1].to_string());
            }
        }
    }

    Ok("Unknown".to_string())
}

fn get_kernel_version() -> Result<String> {
    let output = Command::new("uname")
        .args(&["-r"])
        .output()
        .context("Failed to get kernel version")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_serial_number() -> Result<String> {
    // Try multiple methods to get serial number without root access
    
    // Method 1: Try dmidecode without sudo (may work on some systems)
    if let Ok(output) = Command::new("dmidecode")
        .args(&["--type", "baseboard"])
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            let line = line.trim();
            if line.starts_with("Serial Number:") {
                if let Some(serial_part) = line.split(':').nth(1) {
                    let serial = serial_part.trim();
                    if !serial.is_empty() && serial != "Not Specified" {
                        return Ok(serial.to_string());
                    }
                }
            }
        }
    }
    
    // Method 2: Try reading from /sys/class/dmi/id/board_serial
    if let Ok(serial) = std::fs::read_to_string("/sys/class/dmi/id/board_serial") {
        let serial = serial.trim();
        if !serial.is_empty() && serial != "Not Specified" {
            return Ok(serial.to_string());
        }
    }
    
    // Method 3: Try reading from /sys/class/dmi/id/product_serial
    if let Ok(serial) = std::fs::read_to_string("/sys/class/dmi/id/product_serial") {
        let serial = serial.trim();
        if !serial.is_empty() && serial != "Not Specified" {
            return Ok(serial.to_string());
        }
    }
    
    // Method 4: Try lshw without sudo
    if let Ok(output) = Command::new("lshw")
        .args(&["-class", "system", "-short"])
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.contains("computer") || line.contains("system") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 2 {
                    let potential_serial = parts.last().unwrap_or(&"");
                    if potential_serial.len() > 4 && !potential_serial.contains("computer") {
                        return Ok(potential_serial.to_string());
                    }
                }
            }
        }
    }
    
    // Method 5: Try to get machine-id as fallback (not exactly serial but unique)
    if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
        let machine_id = machine_id.trim();
        if !machine_id.is_empty() {
            return Ok(format!("machine-{}", &machine_id[..8])); // Use first 8 chars
        }
    }
    
    // Final fallback
    Ok("Unknown".to_string())
}

impl DisplayInfo {
    pub fn detect() -> Result<Self> {
        let displays = detect_displays()?;
        Ok(DisplayInfo { displays })
    }
}

fn detect_displays() -> Result<Vec<Display>> {
    let _displays: Vec<Display> = Vec::new();
    
    // Try xrandr first (most common)
    if let Ok(xrandr_displays) = detect_displays_xrandr() {
        if !xrandr_displays.is_empty() {
            return Ok(xrandr_displays);
        }
    }
    
    // Fallback to wlr-randr for Wayland
    if let Ok(wlr_displays) = detect_displays_wlr_randr() {
        if !wlr_displays.is_empty() {
            return Ok(wlr_displays);
        }
    }
    
    // Fallback to basic detection
    detect_displays_fallback()
}

fn detect_displays_xrandr() -> Result<Vec<Display>> {
    let output = Command::new("xrandr")
        .args(&["--verbose"])
        .output()
        .context("Failed to run xrandr")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut displays = Vec::new();
    
    let mut current_display: Option<Display> = None;
    
    for line in output_str.lines() {
        let line = line.trim();
        
        // Check for display name
        if line.contains(" connected") {
            // Save previous display if exists
            if let Some(display) = current_display.take() {
                displays.push(display);
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(name) = parts.first() {
                let is_primary = line.contains("primary");
                current_display = Some(Display {
                    name: name.to_string(),
                    resolution: "Unknown".to_string(),
                    refresh_rate: "Unknown".to_string(),
                    color_depth: "Unknown".to_string(),
                    is_primary,
                    brightness: "Unknown".to_string(),
                    rotation: "Normal".to_string(),
                    scale_factor: "1.0".to_string(),
                    color_profile: "Default".to_string(),
                    connection_type: detect_connection_type(name).unwrap_or_else(|| "Unknown".to_string()),
                });
            }
        }
        
        // Check for resolution and refresh rate
        if line.contains("*") && current_display.is_some() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(resolution_part) = parts.first() {
                if let Some(display) = current_display.as_mut() {
                    display.resolution = resolution_part.to_string();
                    
                    // Extract refresh rate
                    for part in &parts {
                        if part.contains("*") {
                            let rate = part.replace("*", "").replace("+", "");
                            if rate.parse::<f64>().is_ok() {
                                display.refresh_rate = format!("{} Hz", rate);
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Check for color depth
        if line.contains("Depth:") && current_display.is_some() {
            if let Some(depth_start) = line.find("Depth:") {
                let depth_part = &line[depth_start + 6..].trim();
                if let Some(space_pos) = depth_part.find(' ') {
                    let depth = &depth_part[..space_pos];
                    if let Some(display) = current_display.as_mut() {
                        display.color_depth = format!("{} bit", depth);
                    }
                }
            }
        }
    }
    
    // Add the last display
    if let Some(display) = current_display {
        displays.push(display);
    }
    
    Ok(displays)
}

fn detect_displays_wlr_randr() -> Result<Vec<Display>> {
    let output = Command::new("wlr-randr")
        .output()
        .context("Failed to run wlr-randr")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut displays = Vec::new();
    
    let mut current_display: Option<Display> = None;
    
    for line in output_str.lines() {
        let line = line.trim();
        
        // Check for display name
        if !line.starts_with(' ') && !line.is_empty() {
            // Save previous display if exists
            if let Some(display) = current_display.take() {
                displays.push(display);
            }
            
            let name = line.split_whitespace().next().unwrap_or("Unknown").to_string();
            current_display = Some(Display {
                name: name.clone(),
                resolution: "Unknown".to_string(),
                refresh_rate: "Unknown".to_string(),
                color_depth: "Unknown".to_string(),
                is_primary: false, // wlr-randr doesn't clearly indicate primary
                brightness: "Unknown".to_string(),
                rotation: "Normal".to_string(),
                scale_factor: "1.0".to_string(),
                color_profile: "Default".to_string(),
                connection_type: detect_connection_type(&name).unwrap_or_else(|| "Unknown".to_string()),
            });
        }
        
        // Check for current mode (indicated by *)
        if line.contains("*") && current_display.is_some() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(resolution_part) = parts.first() {
                if let Some(display) = current_display.as_mut() {
                    display.resolution = resolution_part.to_string();
                    
                    // Extract refresh rate
                    for part in &parts {
                        if part.contains("Hz") {
                            display.refresh_rate = part.to_string();
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // Add the last display
    if let Some(display) = current_display {
        displays.push(display);
    }
    
    Ok(displays)
}

fn detect_displays_fallback() -> Result<Vec<Display>> {
    // Try to get display information from /sys/class/drm
    let drm_path = std::path::Path::new("/sys/class/drm");
    let mut displays = Vec::new();
    
    if drm_path.exists() {
        if let Ok(entries) = std::fs::read_dir(drm_path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.starts_with("card") && name_str.contains("-") {
                    let display_name = name_str.split('-').nth(1).unwrap_or("Unknown").to_string();
                    
                    // Check if this display is connected
                    let status_path = entry.path().join("status");
                    if let Ok(status) = std::fs::read_to_string(&status_path) {
                        if status.trim() == "connected" {
                            displays.push(Display {
                                name: display_name.clone(),
                                resolution: "Unknown".to_string(),
                                refresh_rate: "Unknown".to_string(),
                                color_depth: "Unknown".to_string(),
                                is_primary: displays.is_empty(), // First one is primary
                                brightness: "Unknown".to_string(),
                                rotation: "Normal".to_string(),
                                scale_factor: "1.0".to_string(),
                                color_profile: "Default".to_string(),
                                connection_type: detect_connection_type(&display_name).unwrap_or_else(|| "Unknown".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // If no displays found, create a fallback
    if displays.is_empty() {
        displays.push(Display {
            name: "Display".to_string(),
            resolution: "Unknown".to_string(),
            refresh_rate: "Unknown".to_string(),
            color_depth: "Unknown".to_string(),
            is_primary: true,
            brightness: "Unknown".to_string(),
            rotation: "Normal".to_string(),
            scale_factor: "1.0".to_string(),
            color_profile: "Default".to_string(),
            connection_type: "Unknown".to_string(),
        });
    }
    
    Ok(displays)
}

impl StorageInfo {
    pub fn detect() -> Result<Self> {
        let devices = detect_storage_devices()?;
        let filesystems = detect_filesystems()?;
        Ok(StorageInfo { devices, filesystems })
    }
}

fn detect_storage_devices() -> Result<Vec<StorageDevice>> {
    let mut devices = Vec::new();
    
    // Get block devices using lsblk
    let output = Command::new("lsblk")
        .args(&["-d", "-o", "NAME,SIZE,TYPE,MODEL,SERIAL", "--json"])
        .output();
    
    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&output_str) {
            if let Some(blockdevices) = json_data["blockdevices"].as_array() {
                for device in blockdevices {
                    let name = device["name"].as_str().unwrap_or("Unknown").to_string();
                    let size = device["size"].as_str().unwrap_or("Unknown").to_string();
                    let device_type = device["type"].as_str().unwrap_or("disk").to_string();
                    let model = device["model"].as_str().unwrap_or("Unknown").to_string();
                    let serial = device["serial"].as_str().unwrap_or("Unknown").to_string();
                    
                    // Skip loop devices and other virtual devices
                    if device_type == "disk" && !name.starts_with("loop") {
                        let interface = detect_storage_interface(&name);
                        let storage_type = detect_storage_type(&name, &model);
                        
                        devices.push(StorageDevice {
                            name: format!("/dev/{}", name),
                            model,
                            size,
                            device_type: storage_type,
                            interface,
                            serial,
                            temperature: get_device_temperature(&name),
                            health: get_device_health(&name),
                        });
                    }
                }
            }
        }
    }
    
    // Fallback to basic detection if JSON parsing fails
    if devices.is_empty() {
        devices = detect_storage_devices_fallback()?;
    }
    
    Ok(devices)
}

fn detect_storage_devices_fallback() -> Result<Vec<StorageDevice>> {
    let mut devices = Vec::new();
    
    let output = Command::new("lsblk")
        .args(&["-d", "-o", "NAME,SIZE,TYPE,MODEL"])
        .output()
        .context("Failed to run lsblk")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines().skip(1) { // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0];
            let size = parts[1];
            let device_type = parts[2];
            let model = if parts.len() > 3 { parts[3..].join(" ") } else { "Unknown".to_string() };
            
            if device_type == "disk" && !name.starts_with("loop") {
                let interface = detect_storage_interface(name);
                let storage_type = detect_storage_type(name, &model);
                
                devices.push(StorageDevice {
                    name: format!("/dev/{}", name),
                    model,
                    size: size.to_string(),
                    device_type: storage_type,
                    interface,
                    serial: "Unknown".to_string(),
                    temperature: get_device_temperature(name),
                    health: get_device_health(name),
                });
            }
        }
    }
    
    Ok(devices)
}

fn detect_storage_interface(device_name: &str) -> String {
    if device_name.starts_with("nvme") {
        "NVMe".to_string()
    } else if device_name.starts_with("sd") {
        "SATA/USB".to_string()
    } else if device_name.starts_with("hd") {
        "IDE/PATA".to_string()
    } else if device_name.starts_with("mmc") {
        "eMMC/SD".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn detect_storage_type(device_name: &str, model: &str) -> String {
    let model_lower = model.to_lowercase();
    
    if device_name.starts_with("nvme") {
        "NVMe SSD".to_string()
    } else if model_lower.contains("ssd") || model_lower.contains("solid state") {
        "SSD".to_string()
    } else if model_lower.contains("hdd") || model_lower.contains("hard disk") {
        "HDD".to_string()
    } else if device_name.starts_with("mmc") {
        "eMMC".to_string()
    } else {
        // Try to detect based on device name patterns
        if device_name.starts_with("sd") {
            "Disk".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

fn get_device_temperature(device_name: &str) -> Option<String> {
    // Try to get temperature from various sources
    if device_name.starts_with("nvme") {
        if let Ok(output) = Command::new("nvme")
            .args(&["smart-log", &format!("/dev/{}", device_name)])
            .output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("temperature") {
                    if let Some(temp_part) = line.split(':').nth(1) {
                        return Some(temp_part.trim().to_string());
                    }
                }
            }
        }
    } else {
        // Try smartctl for SATA drives
        if let Ok(output) = Command::new("smartctl")
            .args(&["-A", &format!("/dev/{}", device_name)])
            .output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Temperature_Celsius") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 9 {
                        return Some(format!("{}°C", parts[9]));
                    }
                }
            }
        }
    }
    
    None
}

fn get_device_health(device_name: &str) -> Option<String> {
    // Try smartctl for health status
    if let Ok(output) = Command::new("smartctl")
        .args(&["-H", &format!("/dev/{}", device_name)])
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("SMART overall-health") {
                if let Some(health_part) = line.split(':').nth(1) {
                    return Some(health_part.trim().to_string());
                }
            }
        }
    }
    
    None
}

fn detect_filesystems() -> Result<Vec<Filesystem>> {
    let mut filesystems = Vec::new();
    
    let output = Command::new("df")
        .args(&["-h", "-T"])
        .output()
        .context("Failed to run df command")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines().skip(1) { // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 7 {
            let device = parts[0].to_string();
            let filesystem_type = parts[1].to_string();
            let total_size = parts[2].to_string();
            let used_size = parts[3].to_string();
            let available_size = parts[4].to_string();
            let usage_percent_str = parts[5].replace("%", "");
            let mountpoint = parts[6].to_string();
            
            // Skip special filesystems
            if !device.starts_with("/dev/") && !device.starts_with("tmpfs") {
                continue;
            }
            
            let usage_percent = usage_percent_str.parse::<f32>().unwrap_or(0.0);
            
            filesystems.push(Filesystem {
                device,
                mountpoint,
                filesystem_type,
                total_size,
                used_size,
                available_size,
                usage_percent,
            });
        }
    }
    
    Ok(filesystems)
}

fn detect_connection_type(display_name: &str) -> Option<String> {
    let name_lower = display_name.to_lowercase();
    
    if name_lower.contains("hdmi") {
        Some("HDMI".to_string())
    } else if name_lower.contains("dp") || name_lower.contains("displayport") {
        Some("DisplayPort".to_string())
    } else if name_lower.contains("vga") {
        Some("VGA".to_string())
    } else if name_lower.contains("dvi") {
        Some("DVI".to_string())
    } else if name_lower.contains("usb") {
        Some("USB-C".to_string())
    } else if name_lower.contains("lvds") || name_lower.contains("edp") {
        Some("Internal".to_string())
    } else {
        None
    }
}
