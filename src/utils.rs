use gtk::prelude::*;
use gtk::{MessageDialog, Window};

pub fn show_error_dialog(parent: Option<&Window>, _title: &str, message: &str) {
    let dialog = MessageDialog::new(
        parent,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        message,
    );
    
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
    }
    
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    
    dialog.present();
}

pub fn show_info_dialog(parent: Option<&Window>, _title: &str, message: &str) {
    let dialog = MessageDialog::new(
        parent,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::Ok,
        message,
    );
    
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
    }
    
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    
    dialog.present();
}

pub fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
