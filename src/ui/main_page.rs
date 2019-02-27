use gdk::ContextExt;
use gdk_pixbuf::PixbufExt;
use gtk::*;
use std::{
	cell::RefCell,
	fs,
	rc::Rc,
};
use crate::{api::{self, execute}, ui::{title, card}};

pub fn render(window: Rc<RefCell<Window>>, header: &HeaderBar, switcher: &StackSwitcher) -> gtk::Box {
	let cont = gtk::Box::new(Orientation::Vertical, 12);
	cont.set_margin_top(48);
	cont.set_margin_bottom(48);
	cont.set_margin_start(96);
	cont.set_margin_end(96);

	let avatar_path = dirs::cache_dir().unwrap().join("mobydick").join("avatar.png");

	let avatar = DrawingArea::new();
	avatar.set_size_request(32, 32);
	avatar.set_halign(Align::Center);
	avatar.connect_draw(clone!(avatar_path => move |da, g| { // More or less stolen from Fractal (https://gitlab.gnome.org/GNOME/fractal/blob/master/fractal-gtk/src/widgets/avatar.rs)
        use std::f64::consts::PI;
        let width = 32.0f64;
        let height = 32.0f64;

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

        let pb = gdk_pixbuf::Pixbuf::new_from_file_at_scale(avatar_path.clone(), 32, 32, true)
        	.unwrap_or_else(|_| IconTheme::get_default().unwrap().load_icon("avatar-default", 32, IconLookupFlags::empty()).unwrap().unwrap());

        let hpos: f64 = (width - (pb.get_height()) as f64) / 2.0;
        g.set_source_pixbuf(&pb, 0.0, hpos);

        g.rectangle(0.0, 0.0, width, height);
        g.fill();

    	Inhibit(false)
	}));
	header.pack_start(&avatar);
	header.set_custom_title(&*switcher);

	let logout_bt = Button::new_from_icon_name("system-log-out", IconSize::LargeToolbar.into());
	logout_bt.connect_clicked(clone!(window => move |_| {
		crate::logout(window.clone());
	}));
	header.pack_end(&logout_bt);
	header.show_all();

	let search = SearchEntry::new();
	search.set_placeholder_text("Search");
	cont.add(&search);

	let results = gtk::Box::new(Orientation::Vertical, 12);
	results.set_valign(Align::Start);
	cont.add(&results);

	rc!(avatar, results);
	clone!(avatar, results, avatar_path);
	wait!(execute(client!().get("/api/v1/users/users/me")) => |res| {
		let res: Result<api::UserInfo, _> = res.json();
		match res {
			Ok(res) => {
				avatar.borrow().set_tooltip_text(format!("Connected as {}.", res.username).as_ref());

				clone!(avatar_path, avatar);
				wait!(execute(client!().get(&res.avatar.medium_square_crop.unwrap_or_default())) => |avatar_dl| {
					fs::create_dir_all(avatar_path.parent().unwrap()).unwrap();
					let mut avatar_file = fs::File::create(avatar_path.clone()).unwrap();
					avatar_dl.copy_to(&mut avatar_file).unwrap();
					avatar.borrow().queue_draw();
				});
			},
			Err(_) => {
				crate::logout(window.clone());
			}
		}
	});

	search.connect_activate(move |s| {
		let results = results.clone();
		wait!(execute(client!().get("/api/v1/search").query(&api::SearchQuery {
			query: s.get_text().unwrap_or_default()
		})) => |res| {
			update_results(res.json().unwrap(), &results.borrow());
		});
	});

	cont.show_all();
	cont
}

fn update_results(res: api::SearchResult, cont: &gtk::Box) {
	for ch in cont.get_children() {
		cont.remove(&ch);
	}

	if res.artists.is_empty() && res.albums.is_empty() && res.tracks.is_empty() {
		cont.add(&Label::new("No results. Try something else."));
	}

	if !res.artists.is_empty() {
		cont.add(&title("Artists"));
		for artist in res.artists.clone() {
			cont.add(&*card::render(artist).borrow());
		}
	}

	if !res.albums.is_empty() {
		cont.add(&title("Albums"));
		for album in res.albums.clone() {
			cont.add(&*card::render(album).borrow());
		}
	}

	if !res.tracks.is_empty() {
		cont.add(&title("Songs"));
		for track in res.tracks.clone() {
			cont.add(&*card::render(track).borrow());
		}
	}

	cont.show_all();
}

