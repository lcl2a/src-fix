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

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
    // HARDCODED manga list
    let mangas = vec![
        Manga {
            id: "466".to_string(),
            title: "膽大黨 (超自然武裝噹噹噹) - TEST".to_string(),
            cover: "https://httpbin.org/image/jpeg".to_string(),
            author: "龍幸伸".to_string(),
            artist: String::new(),
            description: "Test description".to_string(),
            url: format!("{}/comic/466", WWW_URL),
            categories: vec!["Action".to_string(), "Comedy".to_string()],
            status: MangaStatus::Ongoing,
            nsfw: MangaContentRating::Safe,
            viewer: MangaViewer::Rtl,
        },
        Manga {
            id: "533".to_string(),
            title: "魔都精兵的奴隸 - TEST".to_string(),
            cover: "https://httpbin.org/image/png".to_string(),
            author: "タカヒロ".to_string(),
            artist: String::new(),
            description: "Test description 2".to_string(),
            url: format!("{}/comic/533", WWW_URL),
            categories: vec!["Fantasy".to_string()],
            status: MangaStatus::Ongoing,
            nsfw: MangaContentRating::Safe,
            viewer: MangaViewer::Rtl,
        },
    ];
    
    Ok(MangaPageResult {
        manga: mangas,
        has_more: false,
    })
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
    // HARDCODED manga listing
    let mangas = vec![
        Manga {
            id: "466".to_string(),
            title: format!("{} - {}", listing.name, "TEST"),
            cover: "https://httpbin.org/image/jpeg".to_string(),
            author: "Test Author".to_string(),
            artist: String::new(),
            description: "Test description".to_string(),
            url: format!("{}/comic/466", WWW_URL),
            categories: vec!["Test".to_string()],
            status: MangaStatus::Ongoing,
            nsfw: MangaContentRating::Safe,
            viewer: MangaViewer::Rtl,
        },
    ];
    
    Ok(MangaPageResult {
        manga: mangas,
        has_more: false,
    })
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
    // HARDCODED manga details
    Ok(Manga {
        id: id.clone(),
        title: format!("Manga {} - TEST", id),
        cover: "https://httpbin.org/image/jpeg".to_string(),
        author: "Test Author".to_string(),
        artist: "Test Artist".to_string(),
        description: "This is a test manga description".to_string(),
        url: format!("{}/comic/{}", WWW_URL, id),
        categories: vec!["Test".to_string(), "Hardcoded".to_string()],
        status: MangaStatus::Ongoing,
        nsfw: MangaContentRating::Safe,
        viewer: MangaViewer::Rtl,
    })
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
    // HARDCODED chapters
    let chapters = vec![
        Chapter {
            id: "7380".to_string(),
            title: "Chapter 1 - TEST".to_string(),
            volume: 1.0,
            chapter: 1.0,
            url: format!("{}/comic/{}/chapter/7380/images/all", WWW_URL, id),
            scanlator: "Test Scan".to_string(),
            lang: "en".to_string(),
            date_updated: 0.0,
        },
        Chapter {
            id: "7381".to_string(),
            title: "Chapter 2 - TEST".to_string(),
            volume: 1.0,
            chapter: 2.0,
            url: format!("{}/comic/{}/chapter/7381/images/all", WWW_URL, id),
            scanlator: "Test Scan".to_string(),
            lang: "en".to_string(),
            date_updated: 0.0,
        },
        Chapter {
            id: "7382".to_string(),
            title: "Chapter 3 - TEST".to_string(),
            volume: 1.0,
            chapter: 3.0,
            url: format!("{}/comic/{}/chapter/7382/images/all", WWW_URL, id),
            scanlator: "Test Scan".to_string(),
            lang: "en".to_string(),
            date_updated: 0.0,
        },
    ];
    
    Ok(chapters)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
    // HARDCODED pages with test images
    let pages = vec![
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
        Page {
            index: 3,
            url: "https://httpbin.org/image/png".to_string(),
            base64: String::new(),
            text: String::new(),
        },
        Page {
            index: 4,
            url: "https://httpbin.org/image/jpeg".to_string(),
            base64: String::new(),
            text: String::new(),
        },
    ];
    
    Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) -> Request {
    // For test images, we don't need to modify anything
    // But keep the function for compatibility
    request
}