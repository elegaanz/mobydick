use gtk::*;
use std::{
	sync::mpsc, cell::RefCell,
	fs, thread, rc::Rc, time::Duration,
};

pub struct NetworkImage {
	pub img: Rc<RefCell<Image>>,
}

impl NetworkImage {
	pub fn new(url: String) -> NetworkImage {
		let image = Rc::new(RefCell::new(
			Image::new_from_icon_name("image-loading", 4)
		));
		let dest_file = url.split("/media/").last().unwrap().replace('/', "-");
		let dest = dirs::cache_dir().unwrap().join(env!("CARGO_PKG_NAME")).join(dest_file.to_string());
		let (tx, rx) = mpsc::channel();
		thread::spawn(clone!(dest => move || {
			fs::create_dir_all(dest.parent().unwrap()).unwrap();
			let mut file = fs::File::create(dest.clone()).unwrap(); // TODO: check if it exists

			reqwest::Client::new()
				.get(&url)
				.send()
				.unwrap()
				.copy_to(&mut file)
				.unwrap();
			tx.send(dest).unwrap();
		}));
		gtk::idle_add(clone!(image => move || { // Check every 0.5s
			match rx.recv_timeout(Duration::from_millis(500)) {
				Err(_) => glib::Continue(true),
				Ok(res) => {
					let pb = gdk_pixbuf::Pixbuf::new_from_file_at_scale(res, 64, 64, true).unwrap();
					image.borrow().set_from_pixbuf(&pb);
					glib::Continue(false)
				}
			}
		}));
		NetworkImage {
			img: image.clone(),
		}
	}
}
