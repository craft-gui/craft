use serde::Deserialize;
use oku::components::ComponentSpecification;
use oku::elements::{Container, ElementStyles, Image, Text};
use oku_core::resource_manager::ResourceIdentifier;
use oku::style::{AlignItems, Display, FlexDirection, JustifyContent};

#[derive(Debug, Clone, Deserialize)]
pub struct AniListResponse {
    pub data: Data,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Data {
    #[serde(rename(deserialize = "Page"))]
    pub page: Page,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    pub media: Vec<Media>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Media {
    pub id: u32,
    pub title: Title,
    #[serde(rename(deserialize = "coverImage"))]
    pub cover_image: CoverImage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Title {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverImage {
    pub large: String,
}

// Query to use in request
pub const QUERY: &str = "
query {
  Page(page: 1, perPage: 10) {
      media(type: ANIME) {
        id
        title {
          romaji
          english
          native
        }
        coverImage {
          large
        }
      }
    }
}
";


pub fn anime_view(media: &Media) -> ComponentSpecification {
    let mut title: &str = "No Name";

    if let Some(native_title) = &media.title.native {
        title = native_title;
    }

    if let Some(romaji_title) = &media.title.romaji {
        title = romaji_title;
    }

    if let Some(english_title) = &media.title.english {
        title = english_title;
    }

    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .gap("20px")
        .push(Image::new(ResourceIdentifier::Url(media.cover_image.large.clone())))
        .push(Text::new(title))
        .component()
}