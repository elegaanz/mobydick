use gtk::*;
use std::{
	rc::Rc,
	cell::RefCell,
};
use crate::{State, api::*, ui::title};

pub fn render(state: State) -> gtk::Box {
	let cont = gtk::Box::new(Orientation::Vertical, 24);
	cont.set_halign(Align::Center);
	cont.set_valign(Align::Center);
	cont.set_size_request(300, -1);
	let title = title("Login");

	let instance = Input::new("Instance URL")
		.with_placeholder("demo.funkwhale.audio");
	let username = Input::new("Username");
	let password = Input::new_password("Password");

	let login_bt = Button::new_with_label("Login");
	login_bt.get_style_context().map(|c| c.add_class("suggested-action"));
	login_bt.set_margin_bottom(48);
	let widgets = Rc::new(RefCell::new((
		instance, username, password
	)));
	login_bt.connect_clicked(clone!(state, widgets => move |_| {
		let mut api_ctx = crate::api::API.lock().unwrap();
		*api_ctx = Some(RequestContext::new(
			widgets.borrow().0.get_text().unwrap()
		));

		let state = state.clone();
		wait!(execute(api_ctx.as_ref().unwrap().post("/api/v1/token/").json(&LoginData {
			username: widgets.borrow().1.get_text().clone().unwrap(),
			password: widgets.borrow().2.get_text().clone().unwrap(),
		})) => |res| {
			let res: Result<LoginInfo, _> = res.json();

			match res {
				Err(_) => crate::show_error(state.clone(), "Somehting went wrong, check your username and password, and the URL of your instance."),
				Ok(res) => {
					if let Some(ref mut client) = *crate::api::API.lock().unwrap() {
						client.auth(res.token.clone());
					}

					let state = state.borrow();
					state.error.set_revealed(false);
					state.stack.add_titled(&crate::ui::main_page::render(&state.header, &{
						let s = StackSwitcher::new();
						s.set_stack(&state.stack);
						s
					}), "main", "Search Music");
					state.stack.set_visible_child_name("main");
			        state.stack.add_titled(&*crate::ui::dl_list::render().borrow(), "downloads", "Downloads");
					state.stack.remove(&state.stack.get_child_by_name("login").unwrap()); // To avoid having a "Login" tab in the header
					state.stack.show_all();
				}
			}
		});
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
