use gtk::prelude::*;

pub mod card;
pub mod dl_list;
pub mod login_page;
pub mod main_page;
pub mod network_image;

fn title(text: &str) -> gtk::Label {
  let lbl = gtk::Label::new(text);
  if let Some(c) = lbl.get_style_context() {
    c.add_class("h2")
  }
  lbl
}
