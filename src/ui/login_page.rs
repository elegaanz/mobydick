use gtk::*;
use std::{
	rc::Rc,
	cell::RefCell,
};
use crate::{State, api, ui::title};


pub fn render(state: State) -> gtk::Box {
	let cont = gtk::Box::new(Orientation::Vertical, 24);
	cont.set_halign(Align::Center);
	cont.set_valign(Align::Center);
	cont.set_size_request(300, -1);
	let title = title("Login");

	let instance = Input::new("Instance URL")
		.with_placeholder("demo.funkwhale.audio")
		.with_default(state.borrow().instance.clone().unwrap_or_default());
	let username = Input::new("Username")
		.with_default(state.borrow().username.clone().unwrap_or_default());
	let password = Input::new_password("Password");

	let login_bt = Button::new_with_label("Login");
	login_bt.get_style_context().map(|c| c.add_class("suggested-action"));
	login_bt.set_margin_bottom(48);
	let widgets = Rc::new(RefCell::new((
		instance, username, password
	)));
	login_bt.connect_clicked(clone!(state, widgets => move |_| {
		state.borrow_mut().instance = widgets.borrow().0.get_text();
		state.borrow_mut().username = widgets.borrow().1.get_text();
		state.borrow_mut().password = widgets.borrow().2.get_text();

		let res: api::LoginInfo = reqwest::Client::new()
			.post(&format!("https://{}/api/v1/token/", state.borrow().instance.clone().unwrap()))
			.json(&api::LoginData {
				username: state.borrow().username.clone().unwrap(),
				password: state.borrow().password.clone().unwrap(),
			})
			.send()
			.unwrap()
			.json()
			.unwrap();

		state.borrow_mut().token = Some(res.token);
		let main_page = crate::ui::main_page::render(state.clone());
		main_page.show_all();
        state.borrow_mut().stack.add_named(&main_page, "main");
    	state.borrow_mut().stack.set_visible_child(&main_page);
    	state.borrow_mut().stack.show_all();
	}));

	cont.add(&title);
	cont.add(&widgets.borrow().0.render());
	cont.add(&widgets.borrow().1.render());
	cont.add(&widgets.borrow().2.render());
	cont.add(&login_bt);

	cont.show_all();
	cont
}

struct Input<'a> {
	label: &'a str,
	entry: gtk::Entry,
}

impl<'a> Input<'a> {
	fn new(text: &'a str) -> Input {
		let entry = gtk::Entry::new();
		Input {
			label: text,
			entry
		}
	}

	fn new_password(text: &'a str) -> Input {
		let input = Input::new(text);
		input.entry.set_visibility(false);
		input
	}

	fn with_placeholder(self, ph: &'a str) -> Input {
		self.entry.set_placeholder_text(ph);
		self
	}

	fn with_default<S: AsRef<str>>(self, def: S) -> Input<'a> {
		self.entry.set_text(def.as_ref());
		self
	}

	fn get_text(&self) -> Option<String> {
		self.entry.get_text()
	}

	fn render(&self) -> gtk::Box {
		let label = gtk::Label::new(self.label);
		label.set_halign(Align::Start);

		let cont = gtk::Box::new(gtk::Orientation::Vertical, 6);
		cont.add(&label);
		cont.add(&self.entry);
		cont
	}
}
