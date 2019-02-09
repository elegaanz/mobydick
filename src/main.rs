use gtk::{self, prelude::*, *};
use std::{
	collections::HashMap,
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
	error: InfoBar,
	header: HeaderBar,
}

pub type State = Rc<RefCell<AppState>>;

#[derive(Debug, Clone, PartialEq)]
pub enum DlStatus {
	Planned,
	Started,
	Done,
	Cancelled,
}

#[derive(Debug, Clone)]
pub struct Download {
	url: String,
	status: DlStatus,
	output: PathBuf,
	track: api::Track,
}

impl Download {
	pub fn ended(&mut self, out: PathBuf) {
		self.status = DlStatus::Done;
		self.output = out;
	}
}

lazy_static::lazy_static! {
	static ref DOWNLOADS: Arc<Mutex<HashMap<i32, Download>>> = Arc::new(Mutex::new(HashMap::new()));

	static ref DL_JOBS: workerpool::Pool<TrackDl> = workerpool::Pool::new(5);
}

#[derive(Default)]
struct TrackDl;

impl workerpool::Worker for TrackDl {
	type Input = Download;
	type Output = ();

	fn execute(&mut self, dl: Self::Input) -> Self::Output {
		if dl.status == DlStatus::Cancelled {
			return;
		}

		{
			let mut dls = DOWNLOADS.lock().unwrap();
			let mut dl = dls.get_mut(&dl.track.id).unwrap();
			dl.status = DlStatus::Started;
		}

		let mut res = client!().get(&dl.url).send().unwrap();

		let ext = res.headers()
			.get(reqwest::header::CONTENT_DISPOSITION).and_then(|h| h.to_str().ok())
			.unwrap_or(".mp3")
			.rsplitn(2, ".").next().unwrap_or("mp3");

		fs::create_dir_all(dl.output.clone().parent().unwrap()).unwrap();
		let mut out = dl.output.clone();
		out.set_extension(ext);
		let mut file = fs::File::create(out.clone()).unwrap();

		res.copy_to(&mut file).unwrap();

		let mut dls = DOWNLOADS.lock().unwrap();
		if let Some(dl) = dls.get_mut(&dl.track.id) {
			dl.ended(out);
		}
	}
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_icon_from_file("icons/128.svg");
    window.set_title("Funkload");
    window.set_default_size(1080, 720);

	let connected = fs::read(dirs::config_dir().unwrap().join("mobydick").join("data.json")).ok().and_then(|f| {
		let json: serde_json::Value = serde_json::from_slice(&f).ok()?;
		let mut api_ctx = crate::api::API.lock().ok()?;
		let mut ctx = api::RequestContext::new(json["instance"].as_str()?.to_string());
		ctx.auth(json["token"].as_str()?.to_string());
		*api_ctx = Some(ctx);

		Some(())
	}).is_some();

    let state = Rc::new(RefCell::new(AppState {
    	stack: {
    		let s = Stack::new();
    		s.set_vexpand(true);
    		s
    	},
    	error: {
    		let error = InfoBar::new();
			error.set_revealed(false);
			error.set_message_type(MessageType::Error);
			error.get_content_area().unwrap().downcast::<gtk::Box>().unwrap().add(&Label::new("Test test"));
			error.set_show_close_button(true);
			error.connect_close(|e| e.set_revealed(false));
			error.connect_response(|e, _| e.set_revealed(false));
			error
    	},
    	header: {
    		let h = HeaderBar::new();
    		h.set_show_close_button(true);
    		h.set_title("Funkload");
    		h
    	},
    }));

    let main_box = gtk::Box::new(Orientation::Vertical, 0);
    main_box.add(&state.borrow().error);
    main_box.add(&state.borrow().stack);

    let scrolled = ScrolledWindow::new(None, None);
    scrolled.add(&main_box);
    window.add(&scrolled);
    window.set_titlebar(&state.borrow().header);
    window.show_all();

    if connected {
        let main_page = ui::main_page::render(&state.borrow().header, &{
			let s = StackSwitcher::new();
			s.set_stack(&state.borrow().stack);
			s
		});
        state.borrow().stack.add_titled(&main_page, "main", "Search Music");
        state.borrow().stack.add_titled(&*ui::dl_list::render().borrow(), "downloads", "Downloads");
    	state.borrow().stack.set_visible_child_name("main");
    } else {
    	let login_page = ui::login_page::render(state.clone());
    	state.borrow().stack.add_named(&login_page, "login");
    }

    window.connect_delete_event(move |_, _| {
        gtk::main_quit();

        fs::write(
        	dirs::config_dir().unwrap().join("mobydick").join("data.json"),
        	serde_json::to_string(&client!().to_json()).unwrap()
        ).unwrap();

        Inhibit(false)
    });

    gtk::main();
}

fn show_error(state: State, msg: &str) {
	let b = state.borrow().error.get_content_area().unwrap().downcast::<gtk::Box>().unwrap();
	for ch in b.get_children() {
		b.remove(&ch);
	}
	b.add(&Label::new(msg));
	state.borrow().error.show_all();
	state.borrow().error.set_revealed(true);
}
