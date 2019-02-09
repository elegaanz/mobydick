use gtk::*;
use std::{thread, sync::mpsc::channel, rc::Rc, cell::RefCell};
use crate::{Download, DlStatus, api, ui::network_image::NetworkImage};

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

	rc!(card);
	if let Some(dl) = model.download_status() {
		match dl.status {
			DlStatus::Done => {
				let open_bt = Button::new_with_label("Play");
				open_bt.set_valign(Align::Center);
				open_bt.set_vexpand(true);
				open_bt.get_style_context().map(|c| c.add_class("suggested-action"));

				let out = dl.output.clone();
				open_bt.connect_clicked(move |_| {
					open::that(out.clone()).unwrap();
					println!("opened file");
				});
				card.borrow().attach(&open_bt, 3, 0, 1, 2);

				let open_bt = Button::new_with_label("View File");
				open_bt.set_valign(Align::Center);
				open_bt.set_vexpand(true);

				let out = dl.output.clone();
				open_bt.connect_clicked(move |_| {
					open::that(out.parent().unwrap().clone()).unwrap();
					println!("opened folder");
				});
				card.borrow().attach(&open_bt, 2, 0, 1, 2);
			},
			DlStatus::Planned | DlStatus::Started => {
				let cancel_bt = Button::new_with_label("Cancel");
				cancel_bt.set_valign(Align::Center);
				cancel_bt.set_vexpand(true);
				cancel_bt.get_style_context().map(|c| c.add_class("destructive-action"));

				let track_id = dl.track.id;
				cancel_bt.connect_clicked(move |_| {
					let mut dls = crate::DOWNLOADS.lock().unwrap();
					let mut dl = dls.get_mut(&track_id).unwrap();
					dl.status = DlStatus::Cancelled;
					println!("Cancelled");
				});
				card.borrow().attach(&cancel_bt, 3, 0, 1, 2);

				if dl.status == DlStatus::Planned {
					sub_text.set_text(format!("{} — Waiting to download", model.subtext()).as_ref());
				} else {
					sub_text.set_text(format!("{} — Download in progress", model.subtext()).as_ref());
				}
			}
			DlStatus::Cancelled => {
				sub_text.set_text(format!("{} — Cancelled", model.subtext()).as_ref());
			}
		}
	} else {
		let dl_bt = Button::new_with_label("Download");
		dl_bt.set_valign(Align::Center);
		dl_bt.set_vexpand(true);
		dl_bt.get_style_context().map(|c| c.add_class("suggested-action"));

		rc!(dl_bt);
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
				if dl_list.is_empty() {	// Nothing to download
					dl_bt.set_label("Not available");
					dl_bt.set_sensitive(false);
				} else {
					clone!(dl_list);
					dl_bt.connect_clicked(move |_| {
						for dl in dl_list.clone() {
							let mut dls = crate::DOWNLOADS.lock().unwrap();
							dls.insert(dl.track.id, dl.clone());

							crate::DL_JOBS.execute(dl);
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

		card.borrow().attach(&*dl_bt.borrow(), 3, 0, 1, 2);
	}

	{
		let card = card.borrow();
		card.attach(&main_text, 1, 0, 1, 1);
		card.attach(&sub_text, 1, 1, 1, 1);
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

	fn download_status(&self) -> Option<Download> {
		None
	}
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

			for track in album.clone().tracks.unwrap_or_default() {
				dls.push(Download {
					url: track.listen_url.clone(),
					output: dirs::audio_dir().unwrap()
						.join(self.name.clone())
						.join(album.title.clone())
						.join(format!("{}.mp3", track.title.clone())),
					status: DlStatus::Planned,
					track: track.clone().into_full(&album),
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
				status: DlStatus::Planned,
				track: track.clone().into_full(&self),
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
			status: DlStatus::Planned,
			track: self.clone(),
		}]
	}

	fn download_status(&self) -> Option<Download> {
		crate::DOWNLOADS.lock().ok()?.get(&self.id).map(|x| x.clone())
	}
}
