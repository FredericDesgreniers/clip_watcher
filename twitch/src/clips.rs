use failure::Error;
use send_request;
use std::collections::HashMap;
use ClientOptions;

/// A series of paginated clips
#[derive(Debug)]
pub struct Clips {
    pub paginated_clips: Vec<PaginatedClips>,
}

impl Clips {
    /// Get clips from broadcaster
    /// Not including an 'after' page in the options will just start from the top clip
    /// Note that they cannot fetched by 'new' because of twitch api limitations
    pub fn from_broadcaster(
        id: &str,
        options: ClipsOptions,
        client_options: &ClientOptions,
    ) -> Result<Self, Error> {
        let mut result = Vec::new();

        let mut paginated_clips =
            PaginatedClips::from_broadcaster(id, options.clone(), client_options)?;

        let mut page = paginated_clips.pagination.clone();

        result.push(paginated_clips);

        while let Some(next_cursor) = page.cursor {
            let mut options = options.clone();
            options.after = Pagination {
                cursor: Some(next_cursor),
            };

            paginated_clips = PaginatedClips::from_broadcaster(id, options, client_options)?;

            page = paginated_clips.pagination.clone();

            result.push(paginated_clips);
        }

        Ok(Self {
            paginated_clips: result,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginatedClips {
    pub data: Vec<Clip>,
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Pagination {
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Clip {
    pub broadcaster_id: String,
    pub created_at: String,
    pub creator_id: String,
    pub embed_url: String,
    pub game_id: String,
    pub id: String,
    pub language: String,
    pub thumbnail_url: String,
    pub title: String,
    pub url: String,
    pub video_id: String,
    pub view_count: u32,
}

#[derive(Clone)]
pub struct ClipsOptions {
    pub per_page: usize,
    pub after: Pagination,
}

impl PaginatedClips {
    pub fn from_broadcaster(
        id: &str,
        options: ClipsOptions,
        client_options: &ClientOptions,
    ) -> Result<Self, Error> {
        let mut endpoint = format!(
            "helix/clips?broadcaster_id={id}&first={per_page}",
            id = id,
            per_page = options.per_page
        );

        if let Some(cursor) = options.after.cursor {
            endpoint.push_str(&format!("&after={}", cursor));
        }

        let result: PaginatedClips =
            serde_json::from_reader(send_request(&endpoint, &client_options)?)?;

        Ok(result)
    }

    pub fn data(&self) -> &Vec<Clip> {
        &self.data
    }
}

pub trait ClipToCreatorMap {
    fn get_creator_map(&self) -> HashMap<String, (u32, u32)>;
}

impl ClipToCreatorMap for Vec<Clip> {
    fn get_creator_map(&self) -> HashMap<String, (u32, u32)> {
        let mut user_map = HashMap::new();

        for clip in self {
            let info = user_map.entry(clip.creator_id.clone()).or_insert((0, 0));
            info.0 += 1;
            info.1 += clip.view_count;
        }

        user_map
    }
}
