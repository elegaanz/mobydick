use gdk_pixbuf::PixbufExt;
use gdk::ContextExt;
use gtk::{self, prelude::*, *};
use std::{
	cell::RefCell,
	rc::Rc,
	ops::Deref,
	sync::Arc,
	fs,
	path::PathBuf,
	thread,
};
use serde_json::json;

mod api;
#[derive(Debug)]
struct AppState {
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

type State = Rc<RefCell<AppState>>;

#[derive(Debug)]
struct Download {
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

	let (token, instance) = fs::read("data.json").ok().and_then(|f|
		serde_json::from_slice(&f).map(|json: serde_json::Value| {
			(json["token"].as_str().map(ToString::to_string), json["instance"].as_str().map(ToString::to_string))
		}).ok()
	).unwrap_or((None, None));

	let err_revealer = Revealer::new();
	let err_label = Label::new("Error");
	err_revealer.add(&err_label);
    let state = Rc::new(RefCell::new(AppState {
    	client: reqwest::Client::new(),
    	token: token,
    	instance: instance,
    	username: None,
    	password: None,
    	search_result: None,
    	stack: Stack::new(),
    	err_revealer: err_revealer,
    	err_label: err_label,
    	downloads: Arc::new(RefCell::new(Vec::new())),
    }));

    let login_page = make_login_page(state.clone());


    state.borrow_mut().stack.add_named(&login_page, "login");
    window.add(&state.borrow_mut().stack);
    window.show_all();

    if state.borrow().instance.is_some() && state.borrow().token.is_some() {
        let main_page = make_main_page(state.clone());
        state.borrow_mut().stack.add_named(&main_page, "main");
        main_page.show_all();
    	state.borrow_mut().stack.set_visible_child(&main_page);
    	println!("visible {:?}", state.borrow_mut().stack.get_visible_child_name());
    	state.borrow_mut().stack.show_all();
    }

	let state = state.clone();
    window.connect_delete_event(move |_, _| {
        gtk::main_quit();

        fs::write("data.json", serde_json::to_string(&json!({
        	"token": state.borrow().token.clone(),
        	"instance": state.borrow().instance.clone()
        })).unwrap()).unwrap();

        Inhibit(false)
    });

    gtk::main();
}

fn make_login_page(state: State) -> gtk::Box {
	let cont = gtk::Box::new(Orientation::Vertical, 12);
	cont.set_margin_top(48);
	cont.set_margin_bottom(48);
	cont.set_margin_start(96);
	cont.set_margin_end(96);

	let title = Label::new("Login");
	title.get_style_context().unwrap().add_class("h2");
	cont.add(&title);

	let instance = Entry::new();
	instance.set_placeholder_text("Instance URL");
	cont.add(&instance);

	let username = Entry::new();
	username.set_placeholder_text("Username");
	cont.add(&username);

	let pwd = Entry::new();
	pwd.set_visibility(false);
	pwd.set_placeholder_text("Password");
	cont.add(&pwd);

	let login_bt = Button::new_with_label("Login");
	let widgets = Rc::new(RefCell::new((
		instance, username, pwd
	)));
	let state = state.clone();
	let widgets = widgets.clone();
	login_bt.connect_clicked(move |_| {
		state.borrow_mut().instance = widgets.borrow().0.get_text();
		state.borrow_mut().username = widgets.borrow().1.get_text();
		state.borrow_mut().password = widgets.borrow().2.get_text();

		let state = state.clone();
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
		let main_page = make_main_page(state.clone());
		main_page.show_all();
        state.borrow_mut().stack.add_named(&main_page, "main");
    	state.borrow_mut().stack.set_visible_child(&main_page);
    	state.borrow_mut().stack.show_all();
	});
	cont.add(&login_bt);

	cont
}

fn make_main_page(state: State) -> gtk::Box {
	let cont = gtk::Box::new(Orientation::Vertical, 12);
	cont.set_margin_top(48);
	cont.set_margin_bottom(48);
	cont.set_margin_start(96);
	cont.set_margin_end(96);

	let avatar_path = dirs::cache_dir().unwrap().join("funkload-avatar.png");
	let user: api::UserInfo = reqwest::Client::new()
		.get(&format!("https://{}/api/v1/users/users/me/", state.borrow().instance.clone().unwrap()))
		.header(reqwest::header::AUTHORIZATION, format!("JWT {}", state.borrow().token.clone().unwrap_or_default()))
		.send()
		.unwrap()
		.json()
		.unwrap();
	if let Some(url) = user.avatar.medium_square_crop {
		let mut avatar_file = fs::File::create(avatar_path.clone()).unwrap();
		reqwest::Client::new()
			.get(&format!("https://{}{}", state.borrow().instance.clone().unwrap(), url))
			.header(reqwest::header::AUTHORIZATION, format!("JWT {}", state.borrow().token.clone().unwrap_or_default()))
			.send()
			.unwrap()
			.copy_to(&mut avatar_file)
			.unwrap();

		let pb = gdk_pixbuf::Pixbuf::new_from_file_at_scale(avatar_path, 128, 128, true).unwrap();
        let avatar = DrawingArea::new();
		avatar.set_size_request(128, 128);
		avatar.get_style_context().map(|c| c.add_class("avatar"));
		avatar.set_halign(Align::Center);
		avatar.connect_draw(move |da, g| { // More or less stolen from Fractal (https://gitlab.gnome.org/GNOME/fractal/blob/master/fractal-gtk/src/widgets/avatar.rs)
            use std::f64::consts::PI;
            let width = 128.0f64;
            let height = 128.0f64;

            g.set_antialias(cairo::Antialias::Best);

            let context = da.get_style_context().unwrap();
            gtk::render_background(&context, g, 0.0, 0.0, width, height);

            g.arc(
                width / 2.0,
                height / 2.0,
                width.min(height) / 2.0,
                0.0,
                2.0 * PI,
            );
            g.clip();

            let hpos: f64 = (width - (pb.get_height()) as f64) / 2.0;
            g.set_source_pixbuf(&pb, 0.0, hpos);

            g.rectangle(0.0, 0.0, width, height);
            g.fill();

        	Inhibit(false)
		});

		cont.add(&avatar);

		let lbl = Label::new(format!("Welcome {}.", user.username).as_ref());
		lbl.get_style_context().map(|c| c.add_class("h1"));
		cont.add(&lbl);
	}

	let search = SearchEntry::new();
	search.set_placeholder_text("Search");
	cont.add(&search);

	let results = Box::new(Orientation::Vertical, 12);
	cont.add(&results);

	let widgets = Rc::new(RefCell::new(
		(search, results)
	));
	let state = state.clone();
	widgets.clone().borrow().0.connect_activate(move |_| {
		let res: api::SearchResult = reqwest::Client::new()
			.get(&format!("https://{}/api/v1/search", state.borrow().instance.clone().unwrap()))
			.header(reqwest::header::AUTHORIZATION, format!("JWT {}", state.borrow().token.clone().unwrap_or_default()))
			.query(&api::SearchQuery {
				query: widgets.borrow().deref().0.get_text().unwrap_or_default()
			})
			.send()
			.unwrap()
			.json()
			.unwrap();

		state.borrow_mut().search_result = Some(res.clone());
		println!("{:#?}", res);
		update_results(state.clone(), &widgets.borrow().1);
	});

	cont
}

fn update_results(state: State, cont: &Box) {
	for ch in cont.get_children() {
		cont.remove(&ch);
	}

	match &state.borrow().search_result {
		Some(res) => {
			if !res.artists.is_empty() {
				cont.add(&title("Artists"));
				for artist in res.artists.clone() {
					cont.add(&Card::new(artist, state.clone()).render());
				}
			}

			if !res.albums.is_empty() {
				cont.add(&title("Albums"));
				for album in res.albums.clone() {
					cont.add(&Card::new(album, state.clone()).render());
				}
			}

			if !res.tracks.is_empty() {
				cont.add(&title("Songs"));
				for track in res.tracks.clone() {
					cont.add(&Card::new(track, state.clone()).render());
				}
			}
		},
		None => {
			cont.add(&Label::new("Try to search something"));
		}
	}
	cont.show_all();
}

struct Card<T: CardModel> {
	model: T,
	state: State,
}

impl<T: 'static> Card<T> where T: CardModel {
	fn new(model: T, state: State) -> Card<T> {
		Card {
			model,
			state,
		}
	}

	fn render(&self) -> Grid {
		let card = Grid::new();
		if let Some(url) = self.model.image_url() {
			// TODO
		}

		let main_text = Label::new(self.model.text().as_ref());
		main_text.get_style_context().map(|c| c.add_class("h3"));
		main_text.set_hexpand(true);
		main_text.set_halign(Align::Start);
		let sub_text = Label::new(self.model.subtext().as_ref());
		sub_text.get_style_context().map(|c| c.add_class("dim-label"));
		sub_text.set_hexpand(true);
		sub_text.set_halign(Align::Start);

		let dl_bt = Button::new_from_icon_name("go-down", 32);
		dl_bt.set_label("Download");
		let state = self.state.clone();
		let model = self.model.clone();
		dl_bt.connect_clicked(move |_| {
			let downloads = state.borrow().downloads.clone();
			for dl in model.downloads(state.clone()) {
				let token = state.borrow().token.clone().unwrap_or_default();
				thread::spawn(move || {
					fs::create_dir_all(dl.output.clone().parent().unwrap()).unwrap();
					let mut file = fs::File::create(dl.output.clone()).unwrap();
					println!("saving {} in {:?}", dl.url.clone(), dl.output.clone());
					reqwest::Client::new()
						.get(&dl.url)
						.header(reqwest::header::AUTHORIZATION, format!("JWT {}", token.clone()))
						.query(&[( "jwt", token )])
						.send()
						.unwrap()
						.copy_to(&mut file)
						.unwrap();
					println!("saved {:?}", dl.output);
				});
			}
		});

		card.attach(&main_text, 0, 0, 1, 1);
		card.attach(&sub_text, 0, 1, 1, 1);
		card.attach(&dl_bt, 1, 0, 2, 1);

		card
	}
}

trait CardModel: Clone {
	fn text(&self) -> String;
	fn subtext(&self) -> String {
		String::new()
	}
	fn image_url(&self) -> Option<String> {
		None
	}

	fn downloads(&self, state: State) -> Vec<Download>;
}

impl CardModel for api::Artist {
	fn text(&self) -> String {
		self.name.clone()
	}

	fn subtext(&self) -> String {
		format!("{} albums", self.albums.clone().unwrap().len())
	}

	fn downloads(&self, state: State) -> Vec<Download> {
		let mut dls = vec![];
		for album in self.albums.clone().unwrap_or_default() {
			let album: api::Album = reqwest::Client::new()
				.get(&format!("https://{}/api/v1/albums/{}/", state.borrow().instance.clone().unwrap(), album.id))
				.header(reqwest::header::AUTHORIZATION, format!("JWT {}", state.borrow().token.clone().unwrap_or_default()))
				.send()
				.unwrap()
				.json()
				.unwrap();

			for track in album.tracks.unwrap_or_default() {
				let upload = match api::Upload::get_for_track(track.id, state.borrow().instance.clone().unwrap(), state.borrow().token.clone().unwrap()) {
					Some(u) => u,
					_ => continue,
				};
				dls.push(Download {
					url: format!("https://{}{}", state.borrow().instance.clone().unwrap(), upload.listen_url),
					output: dirs::audio_dir().unwrap().join(self.name.clone()).join(album.title.clone()).join(format!("{}.{}", track.title.clone(), upload.extension)),
					done: false,
				});
			}
		}
		dls
	}
}

impl CardModel for api::Album {
	fn text(&self) -> String {
		self.title.clone()
	}

	fn subtext(&self) -> String {
		format!("{} tracks, by {}", self.tracks.clone().map(|t| t.len()).unwrap_or_default(), self.artist.name)
	}

	fn downloads(&self, state: State) -> Vec<Download> {
		self.tracks.clone().unwrap_or_default().iter().filter_map(|track|
			api::Upload::get_for_track(track.id, state.borrow().instance.clone().unwrap(), state.borrow().token.clone().unwrap()).map(|u| Download {
				url: format!("https://{}{}", state.borrow().instance.clone().unwrap(), u.listen_url),
				output: dirs::audio_dir().unwrap().join(self.artist.name.clone()).join(self.title.clone()).join(format!("{}.{}", track.title.clone(), u.extension)),
				done: false,
			})
		).collect()
	}
}

impl CardModel for api::Track {
	fn text(&self) -> String {
		self.title.clone()
	}

	fn subtext(&self) -> String {
		format!("By {}, in {}", self.artist.name, self.album.title)
	}

	fn downloads(&self, state: State) -> Vec<Download> {
		println!("yoy");
		let upload = match api::Upload::get_for_track(self.id, state.borrow().instance.clone().unwrap(), state.borrow().token.clone().unwrap()) {
			Some(u) => u,
			_ => return vec![]
		};
		println!("yay");
		vec![Download {
			url: format!("https://{}{}", state.borrow().instance.clone().unwrap(), upload.listen_url),
			output: dirs::audio_dir().unwrap().join(self.artist.name.clone()).join(self.album.title.clone()).join(format!("{}.{}", self.title.clone(), upload.extension)),
			done: false,
		}]
	}
}

fn title(text: &str) -> Label {
	let lbl = Label::new(text);
	lbl.get_style_context().map(|c| c.add_class("h2"));
	lbl
}
