use crate::api::execute;
use gtk::{Image, ImageExt};
use std::{cell::RefCell, fs, rc::Rc};

pub struct NetworkImage {
  pub img: Rc<RefCell<Image>>,
}

impl NetworkImage {
  pub fn new(url: String) -> NetworkImage {
    let image = Image::new_from_icon_name("image-loading", 4);
    rc!(image);

    let dest_file = url.split("/media/").last().unwrap().replace('/', "-");
    let dest = dirs::cache_dir()
      .unwrap()
      .join(env!("CARGO_PKG_NAME"))
      .join(dest_file);

    if dest.exists() {
      let pb = gdk_pixbuf::Pixbuf::new_from_file_at_scale(dest, 64, 64, true).unwrap();
      image.borrow().set_from_pixbuf(&pb);
    } else {
      clone!(image);
      wait!(execute(client!().get(&url)) => |res| {
          fs::create_dir_all(dest.parent().unwrap()).unwrap();
          let mut file = fs::File::create(dest.clone()).unwrap();
          res.copy_to(&mut file).unwrap();

          let pb = gdk_pixbuf::Pixbuf::new_from_file_at_scale(dest.clone(), 64, 64, true).unwrap();
          image.borrow().set_from_pixbuf(&pb);
      });
    }

    NetworkImage { img: image }
  }
}
