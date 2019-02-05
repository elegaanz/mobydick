use gtk::prelude::*;

pub mod card;
pub mod login_page;
pub mod main_page;
pub mod network_image;

fn title(text: &str) -> gtk::Label {
	let lbl = gtk::Label::new(text);
	lbl.get_style_context().map(|c| c.add_class("h2"));
	lbl
}
