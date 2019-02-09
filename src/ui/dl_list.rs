use gtk::{prelude::*, *};
use std::{cell::RefCell, rc::Rc};
use crate::{ui::card};

pub fn render() -> Rc<RefCell<gtk::Box>> {
	let cont = gtk::Box::new(Orientation::Vertical, 12);
	cont.set_valign(Align::Start);
	cont.set_margin_top(48);
	cont.set_margin_bottom(48);
	cont.set_margin_start(96);
	cont.set_margin_end(96);

	let active = crate::DL_JOBS.active_count();
	rc!(cont, active);
	gtk::idle_add(clone!(cont => move || {
		let active_now = crate::DL_JOBS.active_count();
		if active_now != *active.borrow() {
			*active.borrow_mut() = active_now;

			let cont = cont.borrow();
			for ch in cont.get_children() {
				cont.remove(&ch);
			}

			let dl_list = {
				crate::DOWNLOADS.lock().unwrap().clone()
			};
			for (_, dl) in dl_list {
				cont.add(&*card::render(dl.track).borrow());
			}
			cont.show_all();
		}
		glib::Continue(true)
	}));
	cont.borrow().show_all();
	cont
}
