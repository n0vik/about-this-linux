use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Box, Button, Entry, FileChooserDialog, HeaderBar,
    Image, Label, Orientation, ResponseType, ScrolledWindow, Stack, StackSwitcher,
};
use gdk_pixbuf::Pixbuf;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::config::Config;
use crate::main_window::MainWindow;
use crate::system_info::SystemInfo;
use crate::{AUTHOR, AUTHOR_EMAIL, VERSION, ORIGINAL_AUTHOR, ORIGINAL_REPO, CURRENT_REPO};

pub struct ConfiguratorWindow {
    window: ApplicationWindow,
    config_path: PathBuf,
    config: Rc<RefCell<Config>>,
    preview_image: Rc<RefCell<Option<Image>>>,
}

impl ConfiguratorWindow {
    pub fn new(app: &Application, config_path: PathBuf) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("About this Linux - Configuration")
            .default_width(700)
            .default_height(500)
            .resizable(true)
            .build();

        let config = Rc::new(RefCell::new(Config::default()));
        let preview_image = Rc::new(RefCell::new(None));

        let configurator = ConfiguratorWindow {
            window,
            config_path,
            config,
            preview_image,
        };

        configurator.build_ui();
        configurator
    }

    fn build_ui(&self) {
        // Create header bar
        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(true);
        self.window.set_titlebar(Some(&header_bar));

        // Create stack and stack switcher (same as main window)
        let stack = Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        stack.set_transition_duration(500);

        let stack_switcher = StackSwitcher::new();
        stack_switcher.set_stack(Some(&stack));
        header_bar.set_title_widget(Some(&stack_switcher));

        // Create configuration tabs
        self.create_system_detection_tab(&stack);
        self.create_manual_config_tab(&stack);
        self.create_preview_tab(&stack);
        self.create_about_tab(&stack);

        self.window.set_child(Some(&stack));
    }

    fn create_system_detection_tab(&self, stack: &Stack) {
        let main_box = Box::new(Orientation::Vertical, 20);
        main_box.set_margin_start(40);
        main_box.set_margin_end(40);
        main_box.set_margin_top(40);
        main_box.set_margin_bottom(40);

        let title_label = Label::new(None);
        title_label.set_markup("<span font-size='large'><b>System Information Detection</b></span>");
        title_label.set_halign(gtk::Align::Start);
        main_box.append(&title_label);

        let info_label = Label::new(Some(
            "This will automatically detect your system information using fastfetch and dmidecode.\n\
            Please ensure you have both tools installed before continuing.\n\
            Root access may be required for some hardware information.",
        ));
        info_label.set_halign(gtk::Align::Start);
        main_box.append(&info_label);

        let detect_button = Button::with_label("Detect System Information");
        detect_button.set_halign(gtk::Align::Start);

        let config_clone = self.config.clone();
        let window_clone = self.window.clone();
        detect_button.connect_clicked(move |button| {
            button.set_sensitive(false);
            button.set_label("Detecting...");

            // Run detection in background
            let config_clone = config_clone.clone();
            let button_clone = button.clone();
            let window_clone = window_clone.clone();

            glib::spawn_future_local(async move {
                match SystemInfo::detect() {
                    Ok(system_info) => {
                        // Show file chooser for distro image
                        let file_chooser = FileChooserDialog::new(
                            Some("Select Distro Logo"),
                            Some(&window_clone),
                            gtk::FileChooserAction::Open,
                            &[
                                ("Cancel", ResponseType::Cancel),
                                ("Open", ResponseType::Accept),
                            ],
                        );

                        // Add image filters
                        let filter = gtk::FileFilter::new();
                        filter.set_name(Some("Image files"));
                        filter.add_mime_type("image/*");
                        file_chooser.add_filter(&filter);

                        file_chooser.connect_response(move |dialog, response| {
                            if response == ResponseType::Accept {
                                if let Some(file) = dialog.file() {
                                    if let Some(path) = file.path() {
                                        // Copy image to config directory
                                        let config_dir = ConfiguratorWindow::get_config_dir();
                                        let dest_path = config_dir.join("distro-logo.png");
                                        
                                        if let Err(e) = std::fs::copy(&path, &dest_path) {
                                            eprintln!("Failed to copy image: {}", e);
                                        } else {
                                            let config = system_info.to_config(dest_path.to_string_lossy().to_string());
                                            *config_clone.borrow_mut() = config;
                                        }
                                    } else {
                                        // If no image selected, use tux-logo.png as fallback
                                        let config = system_info.to_config("tux-logo.png".to_string());
                                        *config_clone.borrow_mut() = config;
                                    }
                                } else {
                                    // If no file selected, use tux-logo.png as fallback
                                    let config = system_info.to_config("tux-logo.png".to_string());
                                    *config_clone.borrow_mut() = config;
                                }
                            }
                            dialog.close();
                        });

                        file_chooser.present();
                    }
                    Err(e) => {
                        eprintln!("Failed to detect system info: {}", e);
                        // Show error dialog
                        let dialog = gtk::MessageDialog::new(
                            Some(&window_clone),
                            gtk::DialogFlags::MODAL,
                            gtk::MessageType::Error,
                            gtk::ButtonsType::Ok,
                            &format!("Failed to detect system information: {}", e),
                        );
                        dialog.connect_response(|dialog, _| dialog.close());
                        dialog.present();
                    }
                }

                button_clone.set_sensitive(true);
                button_clone.set_label("Detect System Information");
            });
        });

        main_box.append(&detect_button);

        let next_button = Button::with_label("Next: Manual Configuration");
        next_button.set_halign(gtk::Align::Start);
        
        let stack_clone = stack.clone();
        next_button.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("manual_config");
        });

        main_box.append(&next_button);

        stack.add_titled(&main_box, Some("system_detection"), "Auto Detection");
    }

    fn create_manual_config_tab(&self, stack: &Stack) {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let main_box = Box::new(Orientation::Vertical, 15);
        main_box.set_margin_start(40);
        main_box.set_margin_end(40);
        main_box.set_margin_top(40);
        main_box.set_margin_bottom(40);

        let title_label = Label::new(None);
        title_label.set_markup("<span font-size='large'><b>Manual Configuration</b></span>");
        title_label.set_halign(gtk::Align::Start);
        main_box.append(&title_label);

        // Create entry fields for all configuration options
        let fields = vec![
            ("Hostname/Model", "hostname"),
            ("CPU", "cpu"),
            ("Memory", "memory"),
            ("Startup Disk", "startup_disk"),
            ("Graphics", "graphics"),
            ("Serial Number", "serial_num"),
        ];

        let entries = Rc::new(RefCell::new(std::collections::HashMap::new()));

        for (label_text, field_name) in fields {
            let field_box = Box::new(Orientation::Vertical, 5);
            
            let label = Label::new(Some(label_text));
            label.set_halign(gtk::Align::Start);
            field_box.append(&label);

            let entry = Entry::new();
            entry.set_placeholder_text(Some(&format!("Enter {}", label_text.to_lowercase())));
            field_box.append(&entry);

            entries.borrow_mut().insert(field_name.to_string(), entry);
            main_box.append(&field_box);
        }

        // Image selection
        let image_box = Box::new(Orientation::Vertical, 5);
        let image_label = Label::new(Some("Distro Logo"));
        image_label.set_halign(gtk::Align::Start);
        image_box.append(&image_label);

        let image_button = Button::with_label("Select Image File");
        let config_clone = self.config.clone();
        let preview_image_clone = self.preview_image.clone();
        let window_clone = self.window.clone();

        image_button.connect_clicked(move |_| {
            let file_chooser = FileChooserDialog::new(
                Some("Select Distro Logo"),
                Some(&window_clone),
                gtk::FileChooserAction::Open,
                &[
                    ("Cancel", ResponseType::Cancel),
                    ("Open", ResponseType::Accept),
                ],
            );

            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Image files"));
            filter.add_mime_type("image/*");
            file_chooser.add_filter(&filter);

            let config_clone = config_clone.clone();
            let preview_image_clone = preview_image_clone.clone();

            file_chooser.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let config_dir = ConfiguratorWindow::get_config_dir();
                            let dest_path = config_dir.join("distro-logo.png");
                            
                            if let Err(e) = std::fs::copy(&path, &dest_path) {
                                eprintln!("Failed to copy image: {}", e);
                            } else {
                                config_clone.borrow_mut().distro_image_path = dest_path.to_string_lossy().to_string();
                                
                                // Update preview image
                                if let Ok(pixbuf) = Pixbuf::from_file_at_scale(&dest_path, 100, 100, true) {
                                    let image = Image::from_pixbuf(Some(&pixbuf));
                                    *preview_image_clone.borrow_mut() = Some(image);
                                }
                            }
                        }
                    }
                }
                dialog.close();
            });

            file_chooser.present();
        });

        image_box.append(&image_button);
        main_box.append(&image_box);

        // Save button
        let save_box = Box::new(Orientation::Horizontal, 10);
        let save_button = Button::with_label("Save Configuration");
        save_button.set_css_classes(&["suggested-action"]);

        let entries_clone = entries.clone();
        let config_clone = self.config.clone();
        let config_path = self.config_path.clone();
        let window_clone = self.window.clone();

        save_button.connect_clicked(move |_| {
            // Update config with manual entries
            let mut config = config_clone.borrow_mut();
            let entries = entries_clone.borrow();

            if let Some(entry) = entries.get("hostname") {
                config.hostname = entry.text().to_string();
            }
            if let Some(entry) = entries.get("cpu") {
                config.cpu = entry.text().to_string();
            }
            if let Some(entry) = entries.get("memory") {
                config.memory = entry.text().to_string();
            }
            if let Some(entry) = entries.get("startup_disk") {
                config.startup_disk = entry.text().to_string();
            }
            if let Some(entry) = entries.get("graphics") {
                config.graphics = entry.text().to_string();
            }
            if let Some(entry) = entries.get("serial_num") {
                config.serial_num = entry.text().to_string();
            }

            // Save configuration
            match config.save(&config_path) {
                Ok(_) => {
                    // Show success and launch main window
                    let app = window_clone.application().unwrap();
                    let main_window = MainWindow::new(&app, config.clone());
                    main_window.present();
                    window_clone.close();
                }
                Err(e) => {
                    eprintln!("Failed to save config: {}", e);
                    let dialog = gtk::MessageDialog::new(
                        Some(&window_clone),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Error,
                        gtk::ButtonsType::Ok,
                        &format!("Failed to save configuration: {}", e),
                    );
                    dialog.connect_response(|dialog, _| dialog.close());
                    dialog.present();
                }
            }
        });

        save_box.append(&save_button);

        let preview_button = Button::with_label("Preview");
        let stack_clone = stack.clone();
        preview_button.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("preview");
        });

        save_box.append(&preview_button);
        main_box.append(&save_box);

        scrolled.set_child(Some(&main_box));
        stack.add_titled(&scrolled, Some("manual_config"), "Manual Config");
    }

    fn create_preview_tab(&self, stack: &Stack) {
        let main_box = Box::new(Orientation::Vertical, 20);
        main_box.set_margin_start(40);
        main_box.set_margin_end(40);
        main_box.set_margin_top(40);
        main_box.set_margin_bottom(40);

        let title_label = Label::new(None);
        title_label.set_markup("<span font-size='large'><b>Configuration Preview</b></span>");
        title_label.set_halign(gtk::Align::Start);
        main_box.append(&title_label);

        // Create a preview area that mimics the main window overview
        let preview_box = Box::new(Orientation::Horizontal, 60);
        preview_box.set_halign(gtk::Align::Center);

        // Placeholder for image
        let image_placeholder = Label::new(Some("Image will\nappear here\nafter selection"));
        image_placeholder.set_halign(gtk::Align::Center);
        image_placeholder.set_valign(gtk::Align::Start);
        preview_box.append(&image_placeholder);

        // Info section
        let info_box = Box::new(Orientation::Vertical, 20);
        info_box.set_valign(gtk::Align::Start);

        let preview_distro = Label::new(Some("Distro name will appear here"));
        preview_distro.set_halign(gtk::Align::Start);
        info_box.append(&preview_distro);

        let preview_version = Label::new(Some("Version will appear here"));
        preview_version.set_halign(gtk::Align::Start);
        info_box.append(&preview_version);

        let preview_info = Label::new(Some("System information will appear here"));
        preview_info.set_halign(gtk::Align::Start);
        info_box.append(&preview_info);

        preview_box.append(&info_box);
        main_box.append(&preview_box);

        let back_button = Button::with_label("Back to Configuration");
        let stack_clone = stack.clone();
        back_button.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("manual_config");
        });

        main_box.append(&back_button);

        stack.add_titled(&main_box, Some("preview"), "Preview");
    }

    fn create_about_tab(&self, stack: &Stack) {
        let about_box = Box::new(Orientation::Vertical, 20);
        about_box.set_halign(gtk::Align::Center);
        about_box.set_valign(gtk::Align::Center);

        let title_label = Label::new(None);
        title_label.set_markup(&format!("<b>About this Linux v{} - Configuration</b>", VERSION));
        title_label.set_halign(gtk::Align::Center);
        about_box.append(&title_label);

        let author_label = Label::new(None);
        author_label.set_markup(&format!("By <b>{}</b>", AUTHOR));
        author_label.set_halign(gtk::Align::Center);
        about_box.append(&author_label);

        let email_label = Label::new(Some(AUTHOR_EMAIL));
        email_label.set_halign(gtk::Align::Center);
        about_box.append(&email_label);

        let info_label = Label::new(Some(
            "Configuration wizard for About this Linux\n\
            Choose automatic detection or manual configuration\n\
            to set up your system information display.",
        ));
        info_label.set_halign(gtk::Align::Center);
        about_box.append(&info_label);

        let attribution_label = Label::new(None);
        attribution_label.set_markup(&format!(
            "Based on the original <a href='{}'>AboutThisMc</a> by {}\n\
            Rewritten and enhanced by Kamil 'Novik' Nowicki\n\
            Project repository: <a href='{}'>github.com/n0vik/about-this-linux</a>",
            ORIGINAL_REPO, ORIGINAL_AUTHOR, CURRENT_REPO
        ));
        attribution_label.set_halign(gtk::Align::Center);
        about_box.append(&attribution_label);

        stack.add_titled(&about_box, Some("about"), "About");
    }

    fn get_config_dir() -> PathBuf {
        let mut config_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        config_dir.push(".local/share/about-this-linux");
        std::fs::create_dir_all(&config_dir).unwrap_or_default();
        config_dir
    }

    pub fn present(&self) {
        self.window.present();
    }
}
