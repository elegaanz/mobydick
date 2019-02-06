use gdk::ContextExt;
use gdk_pixbuf::PixbufExt;
use gtk::*;
use std::{
	cell::RefCell,
	rc::Rc,
	ops::Deref,
	fs,
};
use crate::{State, api, ui::{title, card::Card}};

pub fn render(state: State) -> gtk::Box {
	let cont = gtk::Box::new(Orientation::Vertical, 12);
	cont.set_margin_top(48);
	cont.set_margin_bottom(48);
	cont.set_margin_start(96);
	cont.set_margin_end(96);

	let avatar_path = dirs::cache_dir().unwrap().join("funkload-avatar.png");
	/*let user: api::UserInfo = reqwest::Client::new()
		.get(&format!("https://{}/api/v1/users/users/me/", state.borrow().instance.clone().unwrap()))
		.header(reqwest::header::AUTHORIZATION, format!("JWT {}", state.borrow().token.clone().unwrap_or_default()))
		.send()
		.unwrap()
		.json()
		.unwrap();
	let pb = match user.avatar.medium_square_crop {
		Some(url) => {
			let mut avatar_file = fs::File::create(avatar_path.clone()).unwrap();
			reqwest::Client::new()
				.get(&format!("https://{}{}", state.borrow().instance.clone().unwrap(), url))
				.header(reqwest::header::AUTHORIZATION, format!("JWT {}", state.borrow().token.clone().unwrap_or_default()))
				.send()
				.unwrap()
				.copy_to(&mut avatar_file)
				.unwrap();
			gdk_pixbuf::Pixbuf::new_from_file_at_scale(avatar_path, 128, 128, true).unwrap()
		},
		None => {
			IconTheme::get_default().unwrap().load_icon("avatar-default", 128, IconLookupFlags::empty()).unwrap().unwrap()
		}
	};

    let avatar = DrawingArea::new();
	avatar.set_size_request(128, 128);
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
	cont.add(&lbl);*/

	let search = SearchEntry::new();
	search.set_placeholder_text("Search");
	cont.add(&search);

	let results = gtk::Box::new(Orientation::Vertical, 12);
	results.set_valign(Align::Start);
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

	cont.show_all();
	cont
}

fn update_results(state: State, cont: &gtk::Box) {
	for ch in cont.get_children() {
		cont.remove(&ch);
	}

	match &state.borrow().search_result {
		Some(res) => {
			if res.artists.is_empty() && res.albums.is_empty() && res.tracks.is_empty() {
				cont.add(&Label::new("No results. Try something else."));
			}

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

