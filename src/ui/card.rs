use gtk::*;
use std::{fs, thread, sync::mpsc::channel, rc::Rc, cell::RefCell};
use crate::{Download, api::{self, execute}, ui::network_image::NetworkImage};

pub fn render<T>(model: T) -> Rc<RefCell<Grid>> where T: CardModel + 'static {
	let card = Grid::new();
	card.set_column_spacing(12);
	card.set_valign(Align::Start);

	if let Some(url) = model.image_url() {
		let img = NetworkImage::new(url);
		card.attach(&*img.img.borrow(), 0, 0, 1, 2);
	}

	let main_text = Label::new(model.text().as_ref());
	main_text.get_style_context().map(|c| c.add_class("h3"));
	main_text.set_hexpand(true);
	main_text.set_halign(Align::Start);
	let sub_text = Label::new(model.subtext().as_ref());
	sub_text.get_style_context().map(|c| c.add_class("dim-label"));
	sub_text.set_hexpand(true);
	sub_text.set_halign(Align::Start);

	let dl_bt = Button::new_with_label("Download");
	dl_bt.set_valign(Align::Center);
	dl_bt.set_vexpand(true);
	dl_bt.get_style_context().map(|c| c.add_class("suggested-action"));

	rc!(dl_bt, card);
	{
		clone!(dl_bt, card);
		wait!({ // Fetch the list of files to download
			let (tx, rx) = channel();
			thread::spawn(move || {
				let dl_list = model.downloads();
				tx.send(dl_list).unwrap();
			});
			rx
		} => | const dl_list | {
			let dl_bt = dl_bt.borrow();
			println!("DLs: {:?}", dl_list);
			if dl_list.is_empty() {	// Nothing to download
				dl_bt.set_label("Not available");
				dl_bt.set_sensitive(false);
			} else {
				clone!(dl_list);
				dl_bt.connect_clicked(move |_| {
					for dl in dl_list.clone() {
						thread::spawn(move || {
							let mut res = client!().get(&dl.url).send().unwrap();

							let ext = res.headers()
								.get(reqwest::header::CONTENT_DISPOSITION).and_then(|h| h.to_str().ok())
								.unwrap_or(".mp3")
								.rsplitn(2, ".").next().unwrap_or("mp3");

							fs::create_dir_all(dl.output.clone().parent().unwrap()).unwrap();
							let mut out = dl.output.clone();
							out.set_extension(ext);
							let mut file = fs::File::create(out).unwrap();

							println!("saving {} in {:?}", dl.url.clone(), dl.output.clone());
							res.copy_to(&mut file).unwrap();
							println!("saved {:?}", dl.output);
						});
					}
				});
			}

			if dl_list.len() > 1 { // Not only one song
				let more_bt = Button::new_with_label("Details");
				more_bt.set_valign(Align::Center);
				more_bt.set_vexpand(true);
				card.borrow().attach(&more_bt, 2, 0, 1, 2);
			}
		});
	}

	{
		let card = card.borrow();
		card.attach(&main_text, 1, 0, 1, 1);
		card.attach(&sub_text, 1, 1, 1, 1);
		card.attach(&*dl_bt.borrow(), 3, 0, 1, 2);
	}

	card
}

pub trait CardModel: Clone + Send + Sync {
	fn text(&self) -> String;
	fn subtext(&self) -> String {
		String::new()
	}
	fn image_url(&self) -> Option<String> {
		None
	}

	fn downloads(&self) -> Vec<Download>;
}

impl CardModel for api::Artist {
	fn text(&self) -> String {
		self.name.clone()
	}

	fn subtext(&self) -> String {
		format!("{} albums", self.albums.clone().unwrap().len())
	}

	fn image_url(&self) -> Option<String> {
		self.albums.clone()?.iter()
			.next()
			.and_then(|album| album.cover.medium_square_crop.clone())
	}

	fn downloads(&self) -> Vec<Download> {
		let mut dls = vec![];
		for album in self.albums.clone().unwrap_or_default() {
			let album: api::Album = client!().get(&format!("/api/v1/albums/{}/", album.id))
				.send().unwrap()
				.json().unwrap();

			for track in album.tracks.unwrap_or_default() {
				dls.push(Download {
					url: track.listen_url.clone(),
					output: dirs::audio_dir().unwrap()
						.join(self.name.clone())
						.join(album.title.clone())
						.join(format!("{}.mp3", track.title.clone())),
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

	fn image_url(&self) -> Option<String> {
		self.cover.medium_square_crop.clone()
	}

	fn downloads(&self) -> Vec<Download> {
		self.tracks.clone().unwrap_or_default().iter().map(|track|
			Download {
				url: track.listen_url.clone(),
				output: dirs::audio_dir().unwrap()
					.join(self.artist.name.clone())
					.join(self.title.clone())
					.join(format!("{}.mp3", track.title.clone())),
				done: false,
			}
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

	fn image_url(&self) -> Option<String> {
		self.album.cover.medium_square_crop.clone()
	}

	fn downloads(&self) -> Vec<Download> {
		vec![Download {
			url: self.listen_url.clone(),
			output: dirs::audio_dir().unwrap()
				.join(self.artist.name.clone())
				.join(self.album.title.clone())
				.join(format!("{}.mp3", self.title.clone())),
			done: false,
		}]
	}
}
