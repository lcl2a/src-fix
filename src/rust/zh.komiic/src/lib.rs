#![no_std]
extern crate alloc;

use alloc::vec;
use aidoku::{
	error::Result,
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, Vec},
	Chapter, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};
use alloc::string::ToString;

const WWW_URL: &str = "https://komiic.com";

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}

mod helper;
mod parser;

const FILTER_CATEGORY: [&str; 38] = [
	"", "1", "3", "4", "5", "6", "7", "8", "10", "11", "2", "12", "13", "14", "15", "16", "17",
	"18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "9", "28", "31", "32", "33", "34",
	"35", "36", "37", "40", "42",
];
const FILTER_STATUS: [&str; 3] = ["", "ONGOING", "END"];
const FILTER_ORDER_BY: [&str; 3] = ["DATE_UPDATED", "VIEWS", "FAVORITE_COUNT"];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut category = String::new();
	let mut status = String::new();
	let mut order_by = String::new();

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				query = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				let index = filter.value.as_int()? as usize;
				match filter.name.as_str() {
					"类型" => {
						category = FILTER_CATEGORY[index].to_string();
					}
					"状态" => {
						status = FILTER_STATUS[index].to_string();
					}
					_ => continue,
				}
			}
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int()? as usize;
				order_by = FILTER_ORDER_BY[index].to_string();
			}
			_ => continue,
		}
	}

	let body = if query.is_empty() {
		helper::gen_category_body_string(category, status, order_by, page)
	} else {
		helper::gen_search_body_string(query.clone())
	};

	let json = helper::get_json(body.clone());
	let data = json.get("data").as_object()?;
	let mangas;

	if query.is_empty() {
		let list = data.get("comicByCategories").as_array()?;
		mangas = parser::parse_manga_list(list);
	} else {
		let data = data.get("searchComicsAndAuthors").as_object()?;
		let list = data.get("comics").as_array()?;
		mangas = parser::parse_manga_list(list);
	};

	Ok(MangaPageResult {
		manga: mangas,
		has_more: query.is_empty(),
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    let mut is_recent_update = false;
    let mut order_by = String::new();

    match listing.name.as_str() {
        "最近更新" => {
            is_recent_update = true;
        }
        "本月热门" => {
            order_by = String::from("MONTH_VIEWS");
        }
        "历史热门" => {
            order_by = String::from("VIEWS");
        }
        _ => return get_manga_list(Vec::new(), page),
    }

    let body = if is_recent_update {
        helper::gen_recent_update_body_string(page)
    } else {
        helper::gen_hot_body_string(order_by, page)
    };

    let json = helper::get_json(body);
    let data = json.get("data").as_object()?;
    
    if is_recent_update {
        let list = data.get("recentUpdate").as_array()?;
        let list_len = list.len();
        let mangas = parser::parse_manga_list(list);
        Ok(MangaPageResult {
            manga: mangas,
            has_more: list_len == 20,
        })
    } else {
        let list = data.get("hotComics").as_array()?;
        let mangas = parser::parse_manga_list(list);
        Ok(MangaPageResult {
            manga: mangas,
            has_more: false,
        })
    }
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let body = helper::gen_id_body_string(id.clone());
	let json = helper::get_json(body);
	let data = json.get("data").as_object()?;
	let data = data.get("comicById").as_object()?;

	Ok(parser::parse_manga(data))
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    // TEST: Return hardcoded chapters for testing
    let test_chapters = vec![
        Chapter {
            id: "7380".to_string(),
            title: "Chapter 1 (Test)".to_string(),
            volume: -1.0,
            chapter: 1.0,
            url: format!("{}/comic/{}/chapter/7380/images/all", WWW_URL, id),
            scanlator: String::new(),
            lang: String::new(),
            date_updated: 0.0,
        },
        Chapter {
            id: "7381".to_string(),
            title: "Chapter 2 (Test)".to_string(),
            volume: -1.0,
            chapter: 2.0,
            url: format!("{}/comic/{}/chapter/7381/images/all", WWW_URL, id),
            scanlator: String::new(),
            lang: String::new(),
            date_updated: 0.0,
        },
    ];
    
    Ok(test_chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
    // TEST: Return hardcoded test images
    let test_pages = vec![
        Page {
            index: 0,
            url: "https://httpbin.org/image/jpeg".to_string(),
            base64: String::new(),
            text: String::new(),
        },
        Page {
            index: 1,
            url: "https://httpbin.org/image/png".to_string(),
            base64: String::new(),
            text: String::new(),
        },
        Page {
            index: 2,
            url: "https://httpbin.org/image/jpeg".to_string(),
            base64: String::new(),
            text: String::new(),
        },
    ];
    
    Ok(test_pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) -> Request {
    let url = request.url().read();
    
    let cookie = defaults_get("cookie")
        .and_then(|v| v.as_string())
        .map(|s| s.read())
        .unwrap_or_default();
    
    let referer = if url.contains("/api/image/") {
        if let (Some(manga_id), Some(chapter_id)) = (extract_manga_id(&url), extract_chapter_id(&url)) {
            format!("{}/comic/{}/chapter/{}/images/all", WWW_URL, manga_id, chapter_id)
        } else {
            WWW_URL.to_string()
        }
    } else {
        WWW_URL.to_string()
    };
    
    let mut request = request
        .header("Referer", &referer)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "image/webp,image/apng,image/*,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9");
    
    if !cookie.is_empty() {
        request = request.header("Cookie", &cookie);
    }
    
    request
}

fn extract_manga_id(url: &str) -> Option<String> {
    url.split('?').nth(1).and_then(|query| {
        query.split('&').find(|param| param.starts_with("mangaId="))
            .map(|param| param[8..].to_string())
    })
}

fn extract_chapter_id(url: &str) -> Option<String> {
    url.split('?').nth(1).and_then(|query| {
        query.split('&').find(|param| param.starts_with("chapterId="))
            .map(|param| param[10..].to_string())
    })
}