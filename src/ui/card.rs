use gtk::*;
use std::{fs, thread};
use crate::{Download, State, api};

pub struct Card<T: CardModel> {
	model: T,
	state: State,
}

impl<T: 'static> Card<T> where T: CardModel {
	pub fn new(model: T, state: State) -> Card<T> {
		Card {
			model,
			state,
		}
	}

	pub fn render(&self) -> Grid {
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

pub trait CardModel: Clone {
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
