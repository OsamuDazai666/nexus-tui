#![allow(dead_code)]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const ANILIST_URL: &str = "https://graphql.anilist.co";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListItem {
    pub id: String,
    pub title: Option<String>,
    pub title_romaji: Option<String>,
    pub cover_url: Option<String>,
    pub banner_url: Option<String>,
    pub synopsis: Option<String>,
    pub score: Option<f32>,
    pub year: Option<i32>,
    pub episodes: Option<u32>,
    pub genres: Vec<String>,
    pub status: Option<String>,
    pub studios: Vec<String>,
    pub trailer_url: Option<String>,
    pub source: Option<String>,
    pub season: Option<String>,
}

// ── Raw GraphQL response shapes ────────────────────────────────────────────

#[derive(Deserialize)]
struct GqlResponse<T> {
    data: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SearchData {
    #[serde(rename = "Page")]
    page: SearchPage,
}

#[derive(Deserialize)]
struct SearchPage {
    media: Vec<RawMedia>,
}

#[derive(Deserialize)]
struct RecData {
    #[serde(rename = "Media")]
    media: RecMedia,
}

#[derive(Deserialize)]
struct RecMedia {
    recommendations: RecConnection,
}

#[derive(Deserialize)]
struct RecConnection {
    nodes: Vec<RecNode>,
}

#[derive(Deserialize)]
struct RecNode {
    #[serde(rename = "mediaRecommendation")]
    media_recommendation: Option<RawMedia>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMedia {
    id: u64,
    title: Option<RawTitle>,
    cover_image: Option<RawCover>,
    banner_image: Option<String>,
    description: Option<String>,
    average_score: Option<u32>,
    season_year: Option<i32>,
    episodes: Option<u32>,
    genres: Option<Vec<String>>,
    status: Option<String>,
    studios: Option<RawStudioConn>,
    trailer: Option<RawTrailer>,
    source: Option<String>,
    season: Option<String>,
}

#[derive(Deserialize)]
struct RawTitle {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCover {
    extra_large: Option<String>,
    large: Option<String>,
}

#[derive(Deserialize)]
struct RawStudioConn {
    nodes: Vec<RawStudio>,
}

#[derive(Deserialize)]
struct RawStudio {
    name: String,
}

#[derive(Deserialize)]
struct RawTrailer {
    id: Option<String>,
    site: Option<String>,
}

// ── Conversion ─────────────────────────────────────────────────────────────

impl From<RawMedia> for AniListItem {
    fn from(r: RawMedia) -> Self {
        let title_romaji = r.title.as_ref().and_then(|t| t.romaji.clone());
        let title = r.title.as_ref().and_then(|t| {
            t.english.clone().or_else(|| t.romaji.clone())
        });
        let cover_url = r.cover_image.as_ref().and_then(|c| {
            c.extra_large.clone().or_else(|| c.large.clone())
        });
        let score = r.average_score.map(|s| s as f32 / 10.0);
        let genres = r.genres.unwrap_or_default();
        let studios = r.studios
            .map(|s| s.nodes.into_iter().map(|n| n.name).collect())
            .unwrap_or_default();
        let trailer_url = r.trailer.and_then(|t| {
            match t.site.as_deref() {
                Some("youtube") => t.id.map(|id| format!("https://youtu.be/{id}")),
                _ => None,
            }
        });

        AniListItem {
            id: r.id.to_string(),
            title,
            title_romaji,
            cover_url,
            banner_url: r.banner_image,
            synopsis: r.description.map(|d| strip_html(&d)),
            score,
            year: r.season_year,
            episodes: r.episodes,
            genres,
            status: r.status,
            studios,
            trailer_url,
            source: r.source,
            season: r.season,
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

pub async fn search_anime(query: &str) -> Result<Vec<AniListItem>> {
    let gql = r#"
    query ($search: String) {
      Page(page: 1, perPage: 25) {
        media(search: $search, type: ANIME, sort: SEARCH_MATCH) {
          id title { romaji english }
          coverImage { extraLarge large }
          bannerImage description averageScore
          seasonYear episodes genres status
          studios { nodes { name } }
          trailer { id site }
          source season
        }
      }
    }"#;

    let body = serde_json::json!({ "query": gql, "variables": { "search": query } });
    let client = reqwest::Client::new();
    let resp: GqlResponse<SearchData> = client
        .post(ANILIST_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let items = resp
        .data
        .map(|d| d.page.media.into_iter().map(AniListItem::from).collect())
        .unwrap_or_default();

    Ok(items)
}

pub async fn fetch_recommendations(anime_id: &str) -> Result<Vec<AniListItem>> {
    let id: u64 = anime_id.parse().unwrap_or(0);
    let gql = r#"
    query ($id: Int) {
      Media(id: $id, type: ANIME) {
        recommendations(page: 1, perPage: 10) {
          nodes {
            mediaRecommendation {
              id title { romaji english }
              coverImage { extraLarge large }
              bannerImage description averageScore
              seasonYear episodes genres status
              studios { nodes { name } }
              trailer { id site }
              source season
            }
          }
        }
      }
    }"#;

    let body = serde_json::json!({ "query": gql, "variables": { "id": id } });
    let client = reqwest::Client::new();
    let resp: GqlResponse<RecData> = client
        .post(ANILIST_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let items = resp
        .data
        .map(|d| {
            d.media
                .recommendations
                .nodes
                .into_iter()
                .filter_map(|n| n.media_recommendation)
                .map(AniListItem::from)
                .collect()
        })
        .unwrap_or_default();

    Ok(items)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("<br>", "\n")
}
