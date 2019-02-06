use gtk::{self, prelude::*, *};
use std::{
	cell::RefCell,
	rc::Rc,
	sync::Arc,
	fs,
	path::PathBuf,
};
use serde_json::json;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

macro_rules! wait {
	($exp:expr => | $res:ident | $then:block) => {
		let rx = $exp;
		gtk::idle_add(move || {
			match rx.try_recv() {
				Err(_) => glib::Continue(true),
				Ok(mut $res) => {
					$then;
					glib::Continue(false)
				},
			}
		})
	}
}

mod api;
mod ui;

#[derive(Debug)]
pub struct AppState {
    instance: Option<String>,
    username: Option<String>,
    password: Option<String>,

    token: Option<String>,

    client: reqwest::Client,

    search_result: Option<api::SearchResult>,

	stack: Stack,
	err_revealer: Revealer,
	err_label: Label,

	downloads: Arc<RefCell<Vec<Download>>>,
}

pub type State = Rc<RefCell<AppState>>;

#[derive(Debug)]
pub struct Download {
	url: String,
	done: bool,
	output: PathBuf,
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Funkload");
    window.set_default_size(1080, 720);

	let (token, instance, user) = fs::read("data.json").ok().and_then(|f|
		serde_json::from_slice(&f).map(|json: serde_json::Value| (
			json["token"].as_str().map(ToString::to_string),
			json["instance"].as_str().map(ToString::to_string),
			json["username"].as_str().map(ToString::to_string),
		)).ok()
	).unwrap_or((None, None, None));

	let err_revealer = Revealer::new();
	let err_label = Label::new("Error");
	err_revealer.add(&err_label);
    let state = Rc::new(RefCell::new(AppState {
    	client: reqwest::Client::new(),
    	token: token,
    	instance: instance,
    	username: user,
    	password: None,
    	search_result: None,
    	stack: Stack::new(),
    	err_revealer: err_revealer,
    	err_label: err_label,
    	downloads: Arc::new(RefCell::new(Vec::new())),
    }));

    let login_page = ui::login_page::render(state.clone());

    state.borrow().stack.add_named(&login_page, "login");
    let scrolled = ScrolledWindow::new(None, None);
    scrolled.add(&state.borrow().stack);
    window.add(&scrolled);
    window.show_all();

    if state.borrow().instance.is_some() && state.borrow().token.is_some() {
        let main_page = ui::main_page::render(state.clone());
        state.borrow().stack.add_named(&main_page, "main");
    	state.borrow().stack.set_visible_child_name("main");
    	// state.borrow_mut().stack.show_all();
    }

    window.connect_delete_event(clone!(state => move |_, _| {
        gtk::main_quit();

        fs::write("data.json", serde_json::to_string(&json!({
        	"token": state.borrow().token.clone(),
        	"instance": state.borrow().instance.clone(),
        	"username": state.borrow().username.clone(),
        })).unwrap()).unwrap();

        Inhibit(false)
    }));

    gtk::main();
}
