use craft::components::ComponentSpecification;
use craft::elements::{Container, ElementStyles, Image, Text};
use craft::resource_manager::ResourceIdentifier;
use craft::style::Unit;
use craft::style::{Display, FlexDirection};
use craft::Color;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AniListResponse {
    pub data: Data,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Data {
    #[serde(rename(deserialize = "Page"))]
    pub page: Page,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Page {
    pub media: Vec<Media>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Media {
    pub id: u32,
    pub title: Title,
    #[serde(rename(deserialize = "coverImage"))]
    pub cover_image: CoverImage,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Title {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CoverImage {
    pub large: String,
}

// Query to use in request
pub const QUERY: &str = "
query {
  Page(page: 1, perPage: 10) {
      media(type: ANIME, sort: TRENDING_DESC) {
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

    if let Some(english_title) = &media.title.english {
        title = english_title;
    }
    
    if let Some(romaji_title) = &media.title.romaji {
        title = romaji_title;
    }

    let cover_image = Image::new(ResourceIdentifier::Url(media.cover_image.large.clone()))
        .width("185px")
        .max_width("185px")
        .height("265px")
        .max_height("265px")
        .border_radius(4.0, 4.0, 4.0, 4.0)
        .border_width("1px", "1px", "1px", "1px")
        .border_color(Color::from_rgb8(150, 150, 150));
    
    let anime_name = Text::new(title)
        .max_width(Unit::Percentage(100.0))
        .font_size(14.0)
        .color(Color::from_rgb8(50, 50, 50));
    
    Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .column_gap("20px")
        .push(cover_image)
        .push(anime_name)
        .width(Unit::Px(185.0))
        .max_height("317px")
        .component()
}
