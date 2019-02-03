use serde_derive::*;

#[derive(Deserialize, Serialize)]
pub struct LoginData {
	pub password: String,
	pub username: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginInfo {
	pub token: String
}

#[derive(Deserialize, Serialize)]
pub struct UserInfo {
	pub username: String,
	pub avatar: Image,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Image {
	pub medium_square_crop: Option<String>,
	pub small_square_crop: Option<String>,
	pub original: Option<String>,
	pub square_crop: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
	pub query: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchResult {
	pub artists: Vec<Artist>,
	pub albums: Vec<Album>,
	pub tracks: Vec<Track>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Artist {
	pub name: String,
	pub albums: Option<Vec<ArtistAlbum>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Album {
	pub title: String,
	pub artist: ArtistPreview,
	pub tracks: Option<Vec<AlbumTrack>>,
	pub cover: Image,
	pub id: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArtistAlbum {
	pub title: String,
	pub tracks_count: i32,
	pub id: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Track {
	pub id: i32,
	pub title: String,
	pub album: Album,
	pub artist: ArtistPreview,
	pub listen_url: String,
	pub uploads: Option<Vec<Upload>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArtistPreview {
	pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlbumTrack {
	pub id: i32,
	pub title: String,
	pub artist: ArtistPreview,
	pub listen_url: String,
	pub uploads: Option<Vec<Upload>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Upload {
	pub extension: String,
	pub listen_url: String,
}

impl Upload {
	pub fn get_for_track(track_id: i32, instance: String, jwt: String) -> Option<Upload> {
		let track: Track = reqwest::Client::new()
			.get(&format!("https://{}/api/v1/tracks/{}/", instance, track_id))
			.header(reqwest::header::AUTHORIZATION, format!("JWT {}", jwt.clone()))
			.send().unwrap()
			.json().unwrap();
		println!("uploads : {:#?}", track);
		track.uploads.unwrap_or_default().into_iter().next()
	}
}
