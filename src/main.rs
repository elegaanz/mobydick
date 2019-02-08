use gtk::{self, prelude::*, *};
use std::{
	cell::RefCell,
	rc::Rc,
	sync::{Arc, Mutex},
	fs,
	path::PathBuf,
};

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
    ($($n:ident),+) => (
    	$( let $n = $n.clone(); )+
    )
}

macro_rules! rc {
	($($n:ident),+) => (
    	$( let $n = std::rc::Rc::new(std::cell::RefCell::new($n)); )+
    )
}

macro_rules! wait {
	($exp:expr => | const $res:ident | $then:block) => {
		let rx = $exp;
		gtk::idle_add(move || {
			match rx.try_recv() {
				Err(_) => glib::Continue(true),
				Ok($res) => {
					$then;
					glib::Continue(false)
				},
			}
		})
	};
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

macro_rules! client {
	() => (crate::api::API.lock().unwrap().as_ref().unwrap())
}

mod api;
mod ui;

#[derive(Debug)]
pub struct AppState {
	stack: Stack,
	err_revealer: Revealer,
	err_label: Label,
	downloads: Arc<RefCell<Vec<Download>>>,
}

pub type State = Rc<RefCell<AppState>>;

#[derive(Debug, Clone)]
pub struct Download {
	url: String,
	done: bool,
	output: PathBuf,
}

lazy_static! {
	static ref DOWNLOADS: Arc<Mutex<Vec<Download>>> = Arc::new(Mutex::new(vec![]));
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Funkload");
    window.set_default_size(1080, 720);

	let connected = fs::read("data.json").ok().and_then(|f| {
		let json: serde_json::Value = serde_json::from_slice(&f).ok()?;
		let mut api_ctx = crate::api::API.lock().ok()?;
		let mut ctx = api::RequestContext::new(json["instance"].as_str()?.to_string());
		ctx.auth(json["token"].as_str()?.to_string());
		*api_ctx = Some(ctx);

		Some(())
	}).is_some();

	let err_revealer = Revealer::new();
	let err_label = Label::new("Error");
	err_revealer.add(&err_label);
    let state = Rc::new(RefCell::new(AppState {
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

    if connected {
        let main_page = ui::main_page::render();
        state.borrow().stack.add_named(&main_page, "main");
    	state.borrow().stack.set_visible_child_name("main");
    }

    window.connect_delete_event(move |_, _| {
        gtk::main_quit();

        fs::write("data.json", serde_json::to_string(&client!().to_json()).unwrap()).unwrap();

        Inhibit(false)
    });

    gtk::main();
}
