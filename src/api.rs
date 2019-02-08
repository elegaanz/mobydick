use serde_derive::*;
use lazy_static::*;
use workerpool::Worker;
use std::{
	sync::{
		Arc,
		mpsc::{channel, Receiver},
		Mutex,
	},
};

lazy_static! {
	pub static ref API: Arc<Mutex<Option<RequestContext>>>
		= Arc::new(Mutex::new(None));

	static ref JOBS: workerpool::Pool<Req>
		= workerpool::Pool::new(5);
}

pub fn execute(req: reqwest::RequestBuilder) -> Receiver<reqwest::Response> {
	let (tx, rx) = channel();
	JOBS.execute_to(tx, req);
	rx
}

pub struct RequestContext {
	token: String,
	instance: String,
	client: reqwest::Client,
}

impl RequestContext {
	pub fn new(instance: String) -> Self {
		RequestContext {
			token: String::new(),
			instance: instance.clone(),
			client: reqwest::Client::new()
		}
	}

	pub fn auth(&mut self, token: String) {
		self.token = token.clone();
	}

	pub fn get<S: AsRef<str>>(&self, url: S) -> reqwest::RequestBuilder {
		self.client
			.get(&format!("https://{}{}", self.instance, url.as_ref()))
			.header(reqwest::header::AUTHORIZATION, format!("JWT {}", self.token))
	}

	/// Warning: no authentication, since it is only used for login
	pub fn post<S: AsRef<str>>(&self, url: S) -> reqwest::RequestBuilder {
		self.client
			.post(&format!("https://{}{}", self.instance, url.as_ref()))
	}

	pub fn to_json(&self) -> serde_json::Value {
		serde_json::json!({
			"token": self.token,
			"instance": self.instance,
		})
	}
}

#[derive(Default)]
pub struct Req;

impl Worker for Req {
	type Input = reqwest::RequestBuilder;
	type Output = reqwest::Response;

	fn execute(&mut self, req: Self::Input) -> Self::Output {
		req.send().expect("Error while sending request")
	}
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ApiResult<T> {
	Ok(T),
	Err
}

impl<T> Into<Result<T, ()>> for ApiResult<T> {
	fn into(self) -> Result<T, ()> {
		match self {
			ApiResult::Ok(t) => Ok(t),
			ApiResult::Err => Err(()),
		}
	}
}

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
	pub cover: Image,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Track {
	pub id: i32,
	pub title: String,
	pub album: Album,
	pub artist: ArtistPreview,
	pub listen_url: String,
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
}

