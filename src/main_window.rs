use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, HeaderBar, Image, Label, Stack, StackSwitcher, Orientation};
use gdk_pixbuf::Pixbuf;

use crate::config::Config;
use crate::system_info::{DynamicSystemInfo, DisplayInfo, StorageInfo};
use crate::{AUTHOR, AUTHOR_EMAIL, VERSION, ORIGINAL_AUTHOR, ORIGINAL_REPO, CURRENT_REPO};

pub struct MainWindow {
    window: ApplicationWindow,
    config: Config,
}

impl MainWindow {
    pub fn new(app: &Application, config: Config) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("About this Linux")
            .default_width(600)
            .default_height(400)
            .resizable(false)
            .build();
        
        // Set window icon using GTK4 method
        // Note: GTK4 handles window icons differently - typically through desktop files
        // For now we'll set the icon name if available
        window.set_icon_name(Some("tux-logo"));

        // Let GTK follow system theme - don't set any theme preferences

        let main_window = MainWindow { window, config };
        main_window.build_ui();
        main_window
    }

    fn build_ui(&self) {
        // Create header bar
        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(true);
        self.window.set_titlebar(Some(&header_bar));

        // Create stack and stack switcher
        let stack = Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        stack.set_transition_duration(500);

        let stack_switcher = StackSwitcher::new();
        stack_switcher.set_stack(Some(&stack));
        header_bar.set_title_widget(Some(&stack_switcher));

        // Create overview tab
        self.create_overview_tab(&stack);
        
        // Create other tabs
        self.create_display_tab(&stack);
        self.create_storage_tab(&stack);
        self.create_support_tab(&stack);
        self.create_service_tab(&stack);

        self.window.set_child(Some(&stack));
    }

    fn create_overview_tab(&self, stack: &Stack) {
        // Main horizontal box with centering
        let center_box = Box::new(Orientation::Vertical, 0);
        center_box.set_halign(gtk::Align::Center);
        center_box.set_valign(gtk::Align::Center);
        center_box.set_hexpand(true);
        center_box.set_vexpand(true);
        
        let main_box = Box::new(Orientation::Horizontal, self.config.logo_space);
        main_box.set_margin_start(self.config.overview_margins[0]);
        main_box.set_margin_end(self.config.overview_margins[1]);
        main_box.set_margin_top(self.config.overview_margins[2]);
        main_box.set_margin_bottom(self.config.overview_margins[3]);
        main_box.set_halign(gtk::Align::Center);
        main_box.set_valign(gtk::Align::Center);

        // Create distro image with fallback logic
        let mut image_loaded = false;
        
        // Try to load the configured image first
        if let Ok(pixbuf) = Pixbuf::from_file_at_scale(
            &self.config.distro_image_path,
            self.config.distro_image_size[0],
            self.config.distro_image_size[1],
            true,
        ) {
            let image = Image::from_pixbuf(Some(&pixbuf));
            image.set_valign(gtk::Align::Start);
            main_box.append(&image);
            image_loaded = true;
        }
        
        // If that fails, try tux-logo.png as fallback
        if !image_loaded {
            let current_dir_path = format!("{}/tux-logo.png", std::env::current_dir().unwrap_or_default().to_string_lossy());
            let tux_paths = vec![
                "tux-logo.png",
                "./tux-logo.png",
                &current_dir_path,
            ];
            
            for tux_path in tux_paths {
                if let Ok(pixbuf) = Pixbuf::from_file_at_scale(
                    tux_path,
                    self.config.distro_image_size[0],
                    self.config.distro_image_size[1],
                    true,
                ) {
                    let image = Image::from_pixbuf(Some(&pixbuf));
                    image.set_valign(gtk::Align::Start);
                    main_box.append(&image);
                    image_loaded = true;
                    break;
                }
            }
        }
        
        // If still no image, show a placeholder
        if !image_loaded {
            let placeholder = Label::new(Some("🐧\nTux\nLogo"));
            placeholder.set_valign(gtk::Align::Start);
            placeholder.set_halign(gtk::Align::Center);
            main_box.append(&placeholder);
        }

        // Create info column
        let info_vbox = Box::new(Orientation::Vertical, self.config.section_space);
        info_vbox.set_valign(gtk::Align::Center);
        info_vbox.set_halign(gtk::Align::Center);

        // Distro info section
        let distro_info_box = Box::new(Orientation::Vertical, 0);
        distro_info_box.set_halign(gtk::Align::Center);

        // Get dynamic distro information
        let dynamic_info = DynamicSystemInfo::detect().unwrap_or_else(|_| DynamicSystemInfo {
            distro_name: "Unknown Linux".to_string(),
            distro_version: "Unknown".to_string(),
            distro_codename: None,
            kernel: "Unknown".to_string(),
        });

        let distro_name = Label::new(None);
        distro_name.set_markup(&dynamic_info.get_distro_markup());
        distro_name.set_halign(gtk::Align::Center);
        distro_info_box.append(&distro_name);

        let distro_ver = Label::new(Some(&dynamic_info.distro_version));
        distro_ver.set_halign(gtk::Align::Center);
        distro_info_box.append(&distro_ver);

        // Add kernel version
        let kernel_label = Label::new(Some(&format!("Kernel {}", dynamic_info.kernel)));
        kernel_label.set_halign(gtk::Align::Center);
        distro_info_box.append(&kernel_label);

        info_vbox.append(&distro_info_box);

        // System info section
        let system_info_box = Box::new(Orientation::Vertical, 0);
        system_info_box.set_halign(gtk::Align::Center);

        // Hostname
        let hostname_label = Label::new(None);
        hostname_label.set_markup(&format!("<b>{}</b>", &self.config.hostname));
        hostname_label.set_halign(gtk::Align::Center);
        system_info_box.append(&hostname_label);

        // System info fields
        let info_fields = vec![
            ("Processor", &self.config.cpu),
            ("Memory", &self.config.memory),
            ("Startup Disk", &self.config.startup_disk),
            ("Graphics", &self.config.graphics),
            ("Serial Number", &self.config.serial_num),
        ];

        for (field_name, field_value) in info_fields {
            let field_box = Box::new(Orientation::Horizontal, 20);
            field_box.set_halign(gtk::Align::Center);

            let name_label = Label::new(None);
            name_label.set_markup(&format!("<b>{}</b>", field_name));
            name_label.set_halign(gtk::Align::Start);
            field_box.append(&name_label);

            let value_label = Label::new(Some(field_value));
            value_label.set_halign(gtk::Align::Start);
            field_box.append(&value_label);

            system_info_box.append(&field_box);
        }

        info_vbox.append(&system_info_box);

        // Buttons section
        let buttons_box = Box::new(Orientation::Horizontal, 10);
        buttons_box.set_halign(gtk::Align::Center);

        let system_report_btn = Button::with_label("System Report...");
        let software_update_btn = Button::with_label("Software Update...");

        // Add button callbacks if commands are configured
        if !self.config.system_info_command.is_empty() {
            let cmd = self.config.system_info_command.clone();
            system_report_btn.connect_clicked(move |_| {
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .spawn();
            });
        }

        if !self.config.software_update_command.is_empty() {
            let cmd = self.config.software_update_command.clone();
            software_update_btn.connect_clicked(move |_| {
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .spawn();
            });
        }

        buttons_box.append(&system_report_btn);
        buttons_box.append(&software_update_btn);

        info_vbox.append(&buttons_box);
        main_box.append(&info_vbox);
        
        center_box.append(&main_box);
        stack.add_titled(&center_box, Some("overview"), "Overview");
    }

    fn create_display_tab(&self, stack: &Stack) {
        let center_wrapper = Box::new(Orientation::Vertical, 0);
        center_wrapper.set_halign(gtk::Align::Center);
        center_wrapper.set_valign(gtk::Align::Center);
        center_wrapper.set_hexpand(true);
        center_wrapper.set_vexpand(true);
        
        let main_display_box = Box::new(Orientation::Vertical, 20);
        main_display_box.set_margin_start(40);
        main_display_box.set_margin_end(40);
        main_display_box.set_margin_top(40);
        main_display_box.set_margin_bottom(40);

        // Get display information
        match DisplayInfo::detect() {
            Ok(display_info) => {
                if display_info.displays.is_empty() {
                    let no_displays_label = Label::new(Some("No displays detected"));
                    no_displays_label.set_halign(gtk::Align::Center);
                    main_display_box.append(&no_displays_label);
                } else {
                    // Title
                    let title = Label::new(None);
                    title.set_markup("<span font-size='large'><b>Display Information</b></span>");
                    title.set_halign(gtk::Align::Start);
                    title.set_margin_bottom(20);
                    main_display_box.append(&title);

                    // Display each monitor
                    for (index, display) in display_info.displays.iter().enumerate() {
                        let display_box = Box::new(Orientation::Vertical, 8);
                        display_box.set_halign(gtk::Align::Start);
                        display_box.set_margin_bottom(20);

                        // Display name with primary indicator
                        let display_name = if display.is_primary {
                            format!("{} (Primary)", display.name)
                        } else {
                            display.name.clone()
                        };
                        
                        let name_label = Label::new(None);
                        name_label.set_markup(&format!("<b>{}</b>", display_name));
                        name_label.set_halign(gtk::Align::Start);
                        display_box.append(&name_label);

                        // Display properties
                        let properties = vec![
                            ("Resolution", &display.resolution),
                            ("Refresh Rate", &display.refresh_rate),
                            ("Color Depth", &display.color_depth),
                            ("Connection Type", &display.connection_type),
                            ("Scale Factor", &display.scale_factor),
                            ("Rotation", &display.rotation),
                            ("Color Profile", &display.color_profile),
                            ("Brightness", &display.brightness),
                        ];

                        for (prop_name, prop_value) in properties {
                            if prop_value != "Unknown" && !prop_value.is_empty() {
                                let prop_box = Box::new(Orientation::Horizontal, 10);
                                prop_box.set_halign(gtk::Align::Start);
                                prop_box.set_margin_start(20);

                                let name_label = Label::new(Some(&format!("{}:", prop_name)));
                                name_label.set_halign(gtk::Align::Start);
                                name_label.set_size_request(120, -1);
                                prop_box.append(&name_label);

                                let value_label = Label::new(Some(prop_value));
                                value_label.set_halign(gtk::Align::Start);
                                prop_box.append(&value_label);

                                display_box.append(&prop_box);
                            }
                        }

                        main_display_box.append(&display_box);

                        // Add separator line if not the last display
                        if index < display_info.displays.len() - 1 {
                            let separator = gtk::Separator::new(Orientation::Horizontal);
                            separator.set_margin_top(10);
                            separator.set_margin_bottom(10);
                            main_display_box.append(&separator);
                        }
                    }
                }
            }
            Err(e) => {
                let error_label = Label::new(Some(&format!("Error detecting displays: {}", e)));
                error_label.set_halign(gtk::Align::Center);
                main_display_box.append(&error_label);
            }
        }

        center_wrapper.append(&main_display_box);
        
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&center_wrapper));
        
        stack.add_titled(&scrolled, Some("display"), "Display");
    }

    fn create_storage_tab(&self, stack: &Stack) {
        let center_wrapper = Box::new(Orientation::Vertical, 0);
        center_wrapper.set_halign(gtk::Align::Center);
        center_wrapper.set_valign(gtk::Align::Center);
        center_wrapper.set_hexpand(true);
        center_wrapper.set_vexpand(true);
        
        let main_storage_box = Box::new(Orientation::Vertical, 20);
        main_storage_box.set_margin_start(40);
        main_storage_box.set_margin_end(40);
        main_storage_box.set_margin_top(40);
        main_storage_box.set_margin_bottom(40);

        // Get storage information
        match StorageInfo::detect() {
            Ok(storage_info) => {
                // Title
                let title = Label::new(None);
                title.set_markup("<span font-size='large'><b>Storage Information</b></span>");
                title.set_halign(gtk::Align::Start);
                title.set_margin_bottom(20);
                main_storage_box.append(&title);

                // Storage devices section
                if !storage_info.devices.is_empty() {
                    let devices_title = Label::new(None);
                    devices_title.set_markup("<b>Storage Devices</b>");
                    devices_title.set_halign(gtk::Align::Start);
                    devices_title.set_margin_bottom(10);
                    main_storage_box.append(&devices_title);

                    for device in &storage_info.devices {
                        let device_box = Box::new(Orientation::Vertical, 8);
                        device_box.set_halign(gtk::Align::Start);
                        device_box.set_margin_bottom(15);
                        device_box.set_margin_start(20);

                        // Device name and type
                        let device_name = Label::new(None);
                        device_name.set_markup(&format!("<b>{}</b> ({})", device.name, device.device_type));
                        device_name.set_halign(gtk::Align::Start);
                        device_box.append(&device_name);

                        // Device properties
                        let device_properties = vec![
                            ("Model", &device.model),
                            ("Size", &device.size),
                            ("Interface", &device.interface),
                            ("Serial", &device.serial),
                        ];

                        for (prop_name, prop_value) in device_properties {
                            if prop_value != "Unknown" && !prop_value.is_empty() {
                                let prop_box = Box::new(Orientation::Horizontal, 10);
                                prop_box.set_halign(gtk::Align::Start);
                                prop_box.set_margin_start(20);

                                let name_label = Label::new(Some(&format!("{}:", prop_name)));
                                name_label.set_halign(gtk::Align::Start);
                                name_label.set_size_request(80, -1);
                                prop_box.append(&name_label);

                                let value_label = Label::new(Some(prop_value));
                                value_label.set_halign(gtk::Align::Start);
                                prop_box.append(&value_label);

                                device_box.append(&prop_box);
                            }
                        }

                        // Temperature and health if available
                        if let Some(ref temp) = device.temperature {
                            let temp_box = Box::new(Orientation::Horizontal, 10);
                            temp_box.set_halign(gtk::Align::Start);
                            temp_box.set_margin_start(20);

                            let temp_name = Label::new(Some("Temperature:"));
                            temp_name.set_halign(gtk::Align::Start);
                            temp_name.set_size_request(80, -1);
                            temp_box.append(&temp_name);

                            let temp_value = Label::new(Some(temp));
                            temp_value.set_halign(gtk::Align::Start);
                            temp_box.append(&temp_value);

                            device_box.append(&temp_box);
                        }

                        if let Some(ref health) = device.health {
                            let health_box = Box::new(Orientation::Horizontal, 10);
                            health_box.set_halign(gtk::Align::Start);
                            health_box.set_margin_start(20);

                            let health_name = Label::new(Some("Health:"));
                            health_name.set_halign(gtk::Align::Start);
                            health_name.set_size_request(80, -1);
                            health_box.append(&health_name);

                            let health_value = Label::new(Some(health));
                            health_value.set_halign(gtk::Align::Start);
                            health_box.append(&health_value);

                            device_box.append(&health_box);
                        }

                        main_storage_box.append(&device_box);
                    }
                }

                if storage_info.devices.is_empty() {
                    let no_storage_label = Label::new(Some("No storage devices detected"));
                    no_storage_label.set_halign(gtk::Align::Center);
                    main_storage_box.append(&no_storage_label);
                }
            }
            Err(e) => {
                let error_label = Label::new(Some(&format!("Error detecting storage: {}", e)));
                error_label.set_halign(gtk::Align::Center);
                main_storage_box.append(&error_label);
            }
        }

        center_wrapper.append(&main_storage_box);
        
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&center_wrapper));

        stack.add_titled(&scrolled, Some("storage"), "Storage");
    }

    fn create_support_tab(&self, stack: &Stack) {
        let support_box = Box::new(Orientation::Vertical, 20);
        support_box.set_halign(gtk::Align::Center);
        support_box.set_valign(gtk::Align::Center);

        let title_label = Label::new(None);
        title_label.set_markup(&format!("<b>About this Linux v{}</b>", VERSION));
        title_label.set_halign(gtk::Align::Center);
        support_box.append(&title_label);

        let author_label = Label::new(None);
        author_label.set_markup(&format!("By <b>{}</b>", AUTHOR));
        author_label.set_halign(gtk::Align::Center);
        support_box.append(&author_label);

        let email_label = Label::new(Some(AUTHOR_EMAIL));
        email_label.set_halign(gtk::Align::Center);
        support_box.append(&email_label);

        let info_label = Label::new(Some("A customizable 'About this Mac' dialog for Linux systems\nBased on the original AboutThisMc project\nRewritten in Rust with GTK4"));
        info_label.set_halign(gtk::Align::Center);
        support_box.append(&info_label);

        let license_label = Label::new(Some("Licensed under GPL-3.0-or-later"));
        license_label.set_halign(gtk::Align::Center);
        support_box.append(&license_label);

        let attribution_label = Label::new(None);
        attribution_label.set_markup(&format!(
            "Based on the original <a href='{}'>AboutThisMc</a> by {}\n\
            Modified and enhanced by Kamil 'Novik' Nowicki\n\
            Current repository: <a href='{}'>github.com/n0vik/about-this-linux</a>",
            ORIGINAL_REPO, ORIGINAL_AUTHOR, CURRENT_REPO
        ));
        attribution_label.set_halign(gtk::Align::Center);
        support_box.append(&attribution_label);

        stack.add_titled(&support_box, Some("support"), "Support");
    }

    fn create_service_tab(&self, stack: &Stack) {
        let main_service_box = Box::new(Orientation::Vertical, 25);
        main_service_box.set_margin_start(40);
        main_service_box.set_margin_end(40);
        main_service_box.set_margin_top(40);
        main_service_box.set_margin_bottom(40);
        main_service_box.set_halign(gtk::Align::Center);
        main_service_box.set_valign(gtk::Align::Center);

        // Get dynamic distro information for service links
        let dynamic_info = DynamicSystemInfo::detect().unwrap_or_else(|_| DynamicSystemInfo {
            distro_name: "Unknown Linux".to_string(),
            distro_version: "Unknown".to_string(),
            distro_codename: None,
            kernel: "Unknown".to_string(),
        });

        // Title
        let title = Label::new(None);
        title.set_markup(&format!("<span font-size='large'><b>{} Service &amp; Support</b></span>", dynamic_info.distro_name));
        title.set_halign(gtk::Align::Center);
        title.set_margin_bottom(20);
        main_service_box.append(&title);

        // Get distribution-specific links
        let service_links = get_distro_service_links(&dynamic_info.distro_name);

        // Create sections for different types of links
        let sections = vec![
            ("Documentation &amp; Help", &service_links.documentation),
            ("Community &amp; Forums", &service_links.community),
            ("Bug Reports &amp; Issues", &service_links.bug_reports),
            ("Download &amp; Updates", &service_links.downloads),
        ];

        for (section_title, links) in sections {
            if !links.is_empty() {
                // Section title
                let section_label = Label::new(None);
                section_label.set_markup(&format!("<b>{}</b>", section_title));
                section_label.set_halign(gtk::Align::Start);
                section_label.set_margin_bottom(10);
                main_service_box.append(&section_label);

                // Links
                let links_box = Box::new(Orientation::Vertical, 8);
                links_box.set_margin_start(20);
                links_box.set_margin_bottom(15);

                for (link_name, url) in links {
                    let link_label = Label::new(None);
                    link_label.set_markup(&format!("• <a href='{}'>{}</a>", url, link_name));
                    link_label.set_halign(gtk::Align::Start);
                    links_box.append(&link_label);
                }

                main_service_box.append(&links_box);
            }
        }

        // Add general Linux resources if no specific distro links
        if service_links.documentation.is_empty() && service_links.community.is_empty() {
            let fallback_label = Label::new(None);
            fallback_label.set_markup("<b>General Linux Resources</b>");
            fallback_label.set_halign(gtk::Align::Start);
            fallback_label.set_margin_bottom(10);
            main_service_box.append(&fallback_label);

            let general_links = vec![
                ("The Linux Documentation Project", "https://tldp.org/"),
                ("Linux.org Community", "https://www.linux.org/"),
                ("StackOverflow Linux", "https://stackoverflow.com/questions/tagged/linux"),
                ("Reddit r/linux", "https://reddit.com/r/linux"),
                ("DistroWatch", "https://distrowatch.com/"),
            ];

            let general_box = Box::new(Orientation::Vertical, 8);
            general_box.set_margin_start(20);

            for (name, url) in general_links {
                let link_label = Label::new(None);
                link_label.set_markup(&format!("• <a href='{}'>{}</a>", url, name));
                link_label.set_halign(gtk::Align::Start);
                general_box.append(&link_label);
            }

            main_service_box.append(&general_box);
        }

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_child(Some(&main_service_box));

        stack.add_titled(&scrolled, Some("service"), "Service");
    }

    pub fn present(&self) {
        self.window.present();
    }
}

#[derive(Debug, Clone)]
struct DistroServiceLinks {
    documentation: Vec<(String, String)>,
    community: Vec<(String, String)>,
    bug_reports: Vec<(String, String)>,
    downloads: Vec<(String, String)>,
}

fn get_distro_service_links(distro_name: &str) -> DistroServiceLinks {
    let distro_lower = distro_name.to_lowercase();
    
    if distro_lower.contains("arch") {
        DistroServiceLinks {
            documentation: vec![
                ("Arch Wiki".to_string(), "https://wiki.archlinux.org/".to_string()),
                ("Installation Guide".to_string(), "https://wiki.archlinux.org/title/Installation_guide".to_string()),
                ("General Recommendations".to_string(), "https://wiki.archlinux.org/title/General_recommendations".to_string()),
            ],
            community: vec![
                ("Arch Linux Forums".to_string(), "https://bbs.archlinux.org/".to_string()),
                ("Reddit r/archlinux".to_string(), "https://reddit.com/r/archlinux".to_string()),
                ("IRC #archlinux".to_string(), "https://wiki.archlinux.org/title/IRC_channels".to_string()),
            ],
            bug_reports: vec![
                ("Bug Tracker".to_string(), "https://bugs.archlinux.org/".to_string()),
                ("Security Issues".to_string(), "https://security.archlinux.org/".to_string()),
            ],
            downloads: vec![
                ("Official Downloads".to_string(), "https://archlinux.org/download/".to_string()),
                ("Package Search".to_string(), "https://archlinux.org/packages/".to_string()),
                ("AUR".to_string(), "https://aur.archlinux.org/".to_string()),
            ],
        }
    } else if distro_lower.contains("ubuntu") {
        DistroServiceLinks {
            documentation: vec![
                ("Ubuntu Documentation".to_string(), "https://help.ubuntu.com/".to_string()),
                ("Community Help Wiki".to_string(), "https://help.ubuntu.com/community".to_string()),
                ("Server Guide".to_string(), "https://ubuntu.com/server/docs".to_string()),
            ],
            community: vec![
                ("Ubuntu Forums".to_string(), "https://ubuntuforums.org/".to_string()),
                ("Ask Ubuntu".to_string(), "https://askubuntu.com/".to_string()),
                ("Reddit r/Ubuntu".to_string(), "https://reddit.com/r/Ubuntu".to_string()),
                ("Ubuntu Discourse".to_string(), "https://discourse.ubuntu.com/".to_string()),
            ],
            bug_reports: vec![
                ("Launchpad Bugs".to_string(), "https://bugs.launchpad.net/ubuntu".to_string()),
                ("Report a Bug".to_string(), "https://help.ubuntu.com/community/ReportingBugs".to_string()),
            ],
            downloads: vec![
                ("Ubuntu Downloads".to_string(), "https://ubuntu.com/download".to_string()),
                ("Package Search".to_string(), "https://packages.ubuntu.com/".to_string()),
                ("Snap Store".to_string(), "https://snapcraft.io/".to_string()),
            ],
        }
    } else if distro_lower.contains("fedora") {
        DistroServiceLinks {
            documentation: vec![
                ("Fedora Documentation".to_string(), "https://docs.fedoraproject.org/".to_string()),
                ("Quick Docs".to_string(), "https://docs.fedoraproject.org/en-US/quick-docs/".to_string()),
                ("Installation Guide".to_string(), "https://docs.fedoraproject.org/en-US/fedora/latest/install-guide/".to_string()),
            ],
            community: vec![
                ("Fedora Discussion".to_string(), "https://discussion.fedoraproject.org/".to_string()),
                ("Ask Fedora".to_string(), "https://ask.fedoraproject.org/".to_string()),
                ("Reddit r/Fedora".to_string(), "https://reddit.com/r/Fedora".to_string()),
                ("Matrix Chat".to_string(), "https://chat.fedoraproject.org/".to_string()),
            ],
            bug_reports: vec![
                ("Bugzilla".to_string(), "https://bugzilla.redhat.com/".to_string()),
                ("Report Issues".to_string(), "https://docs.fedoraproject.org/en-US/quick-docs/howto-file-a-bug/".to_string()),
            ],
            downloads: vec![
                ("Get Fedora".to_string(), "https://getfedora.org/".to_string()),
                ("Package Search".to_string(), "https://packages.fedoraproject.org/".to_string()),
                ("COPR Repos".to_string(), "https://copr.fedorainfracloud.org/".to_string()),
            ],
        }
    } else if distro_lower.contains("debian") {
        DistroServiceLinks {
            documentation: vec![
                ("Debian Documentation".to_string(), "https://www.debian.org/doc/".to_string()),
                ("Debian Wiki".to_string(), "https://wiki.debian.org/".to_string()),
                ("Installation Guide".to_string(), "https://www.debian.org/releases/stable/installmanual".to_string()),
            ],
            community: vec![
                ("Debian User Forums".to_string(), "https://forums.debian.net/".to_string()),
                ("Reddit r/debian".to_string(), "https://reddit.com/r/debian".to_string()),
                ("Mailing Lists".to_string(), "https://lists.debian.org/".to_string()),
                ("IRC Channels".to_string(), "https://wiki.debian.org/IRC".to_string()),
            ],
            bug_reports: vec![
                ("Bug Tracking System".to_string(), "https://bugs.debian.org/".to_string()),
                ("Report a Bug".to_string(), "https://www.debian.org/Bugs/Reporting".to_string()),
            ],
            downloads: vec![
                ("Getting Debian".to_string(), "https://www.debian.org/distrib/".to_string()),
                ("Package Search".to_string(), "https://packages.debian.org/".to_string()),
                ("CD/DVD Images".to_string(), "https://www.debian.org/CD/".to_string()),
            ],
        }
    } else if distro_lower.contains("opensuse") || distro_lower.contains("suse") {
        DistroServiceLinks {
            documentation: vec![
                ("openSUSE Documentation".to_string(), "https://doc.opensuse.org/".to_string()),
                ("openSUSE Wiki".to_string(), "https://en.opensuse.org/Portal:Wiki".to_string()),
                ("Reference Guide".to_string(), "https://doc.opensuse.org/documentation/leap/reference/html/book-reference/index.html".to_string()),
            ],
            community: vec![
                ("openSUSE Forums".to_string(), "https://forums.opensuse.org/".to_string()),
                ("Reddit r/openSUSE".to_string(), "https://reddit.com/r/openSUSE".to_string()),
                ("Discord Server".to_string(), "https://discord.gg/openSUSE".to_string()),
                ("Telegram Groups".to_string(), "https://en.opensuse.org/openSUSE:Telegram".to_string()),
            ],
            bug_reports: vec![
                ("Bugzilla".to_string(), "https://bugzilla.opensuse.org/".to_string()),
                ("How to Report Bugs".to_string(), "https://en.opensuse.org/openSUSE:Submitting_bug_reports".to_string()),
            ],
            downloads: vec![
                ("Get openSUSE".to_string(), "https://get.opensuse.org/".to_string()),
                ("Software Search".to_string(), "https://software.opensuse.org/".to_string()),
                ("Build Service".to_string(), "https://build.opensuse.org/".to_string()),
            ],
        }
    } else if distro_lower.contains("manjaro") {
        DistroServiceLinks {
            documentation: vec![
                ("Manjaro Wiki".to_string(), "https://wiki.manjaro.org/".to_string()),
                ("User Guide".to_string(), "https://manjaro.org/support/userguide/".to_string()),
                ("Installation Guide".to_string(), "https://wiki.manjaro.org/index.php/Installation_Guides".to_string()),
            ],
            community: vec![
                ("Manjaro Forum".to_string(), "https://forum.manjaro.org/".to_string()),
                ("Reddit r/ManjaroLinux".to_string(), "https://reddit.com/r/ManjaroLinux".to_string()),
                ("Telegram Groups".to_string(), "https://t.me/manjarolinux".to_string()),
            ],
            bug_reports: vec![
                ("GitLab Issues".to_string(), "https://gitlab.manjaro.org/".to_string()),
                ("Bug Reports".to_string(), "https://forum.manjaro.org/c/technical-issues-and-assistance/".to_string()),
            ],
            downloads: vec![
                ("Download Manjaro".to_string(), "https://manjaro.org/download/".to_string()),
                ("Package Search".to_string(), "https://packages.manjaro.org/".to_string()),
                ("AUR".to_string(), "https://aur.archlinux.org/".to_string()),
            ],
        }
    } else {
        // Default empty links for unknown distributions
        DistroServiceLinks {
            documentation: vec![],
            community: vec![],
            bug_reports: vec![],
            downloads: vec![],
        }
    }
}
