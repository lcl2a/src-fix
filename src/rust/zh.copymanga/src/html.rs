use crate::{
	json::{EncryptedJson as _, MangaItem, page_list},
	net::Url,
};
use aidoku::{
	Manga, MangaPageResult, MangaStatus, Page, Result, SelectFilter,
	alloc::{String, Vec, borrow::ToOwned as _},
	error,
	imports::{
		html::{Document, Element, ElementList},
	},
};

pub trait GenresPage {
	fn filter(&self) -> Result<SelectFilter>;
}

impl GenresPage for Document {
	fn filter(&self) -> Result<SelectFilter> {
		let (mut options, mut ids) = self
			.try_select("div#all a:not([disabled])")?
			.filter_map(|element| {
				let option = element.own_text()?.into();
				let id = element.attr("href")?.rsplit_once('=')?.1.to_owned().into();
				Some((option, id))
			})
			.collect::<(Vec<_>, Vec<_>)>();

		options.insert(0, "全部".into());
		ids.insert(0, "".into());

		Ok(SelectFilter {
			id: "題材".into(),
			title: Some("題材".into()),
			is_genre: true,
			uses_tag_style: true,
			options,
			ids: Some(ids),
			..Default::default()
		})
	}
}

pub trait FiltersPage {
	fn manga_page_result(&self) -> Result<MangaPageResult>;
}

impl FiltersPage for Document {
	fn manga_page_result(&self) -> Result<MangaPageResult> {
		let js_literal = self
			.try_select_first("div.exemptComic-box")?
			.attr("list")
			.ok_or_else(|| error!("Attribute not found: `list`"))?;
		let json = js_literal_to_json(&js_literal)?;
		let entries = serde_json::from_str::<Vec<MangaItem>>(&json)?
			.into_iter()
			.map(Into::into)
			.collect();

		let has_next_page = !self
			.try_select("li.page-all-item")?
			.next_back()
			.ok_or_else(|| error!("No element found for selector: `li.page-all-item`"))?
			.has_class("active");

		Ok(MangaPageResult {
			entries,
			has_next_page,
		})
	}
}

fn js_literal_to_json(input: &str) -> Result<String> {
	let mut output = String::with_capacity(input.len());
	let mut chars = input.chars().peekable();

	while let Some(ch) = chars.next() {
		match ch {
			'\'' => {
				output.push('"');
				loop {
					let next = chars
						.next()
						.ok_or_else(|| error!("Unterminated single-quoted string"))?;

					match next {
						'\\' => {
							output.push('\\');
							let escaped = chars
								.next()
								.ok_or_else(|| error!("Invalid escape sequence"))?;
							match escaped {
								'\'' => output.push('\''),
								'"' => output.push('"'),
								'\\' => output.push('\\'),
								'/' => output.push('/'),
								'b' => output.push('b'),
								'f' => output.push('f'),
								'n' => output.push('n'),
								'r' => output.push('r'),
								't' => output.push('t'),
								'u' => {
									output.push('u');
									for _ in 0..4 {
										let digit = chars
											.next()
											.ok_or_else(|| error!("Invalid unicode escape"))?;
										if !digit.is_ascii_hexdigit() {
											return Err(error!("Invalid unicode escape"));
										}
										output.push(digit);
									}
								}
								other => output.push(other),
							}
						}
						'\'' => {
							output.push('"');
							break;
						}
						'"' => output.push_str("\\\""),
						'\n' | '\r' => return Err(error!("Unterminated single-quoted string")),
						other => output.push(other),
					}
				}
			}
			'"' => {
				output.push('"');
				loop {
					let next = chars
						.next()
						.ok_or_else(|| error!("Unterminated double-quoted string"))?;

					match next {
						'\\' => {
							output.push('\\');
							let escaped = chars
								.next()
								.ok_or_else(|| error!("Invalid escape sequence"))?;
							output.push(escaped);
						}
						'"' => {
							output.push('"');
							break;
						}
						'\n' | '\r' => return Err(error!("Unterminated double-quoted string")),
						other => output.push(other),
					}
				}
			}
			c if is_identifier_start(c) => {
				let mut identifier = String::new();
				identifier.push(c);

				while let Some(&next) = chars.peek() {
					if is_identifier_continue(next) {
						identifier.push(next);
						chars.next();
					} else {
						break;
					}
				}

				if is_object_key(&chars) {
					output.push('"');
					output.push_str(&identifier);
					output.push('"');
				} else {
					output.push_str(&identifier);
				}
			}
			other => output.push(other),
		}
	}

	Ok(output)
}

fn is_object_key(chars: &core::iter::Peekable<core::str::Chars<'_>>) -> bool {
	let mut lookahead = chars.clone();

	while let Some(next) = lookahead.peek() {
		if next.is_whitespace() {
			lookahead.next();
		} else {
			return *next == ':';
		}
	}

	false
}

fn is_identifier_start(c: char) -> bool {
	c == '_' || c == '$' || c.is_ascii_alphabetic()
}

fn is_identifier_continue(c: char) -> bool {
	is_identifier_start(c) || c.is_ascii_digit()
}

pub trait MangaPage {
	fn update_details(&self, manga: &mut Manga) -> Result<()>;
}

impl MangaPage for Document {
	fn update_details(&self, manga: &mut Manga) -> Result<()> {
		manga.title = simplified_title(self)?;

		manga.cover = self
			.try_select_first("img[data-src]")?
			.attr("data-src")
			.map(|resized| resized.replace(".328x422.jpg", ""));

		let authors = self
			.try_select("span.comicParticulars-right-txt > a")?
			.filter_map(|element| element.text())
			.collect();
		manga.authors = Some(authors);

		manga.description = self.try_select_first("p.intro")?.text();

		manga.url = Url::manga(&manga.key).to_string().ok();

		let tags = self
			.try_select("span.comicParticulars-tag > a")?
			.filter_map(|element| {
				let tag = element.text()?.strip_prefix('#')?.into();
				Some(tag)
			})
			.collect();
		manga.tags = Some(tags);

		manga.status = match self
			.try_select_first("li:contains(狀態：) > span.comicParticulars-right-txt")?
			.text()
			.as_deref()
		{
			Some("連載中") => MangaStatus::Ongoing,
			Some("已完結" | "短篇") => MangaStatus::Completed,
			_ => MangaStatus::Unknown,
		};

		Ok(())
	}
}

pub fn simplified_title(document: &Document) -> Result<String> {
	document
		.try_select_first("li:contains(別名：) > p.comicParticulars-right-txt")?
		.text()
		.or_else(|| {
			document
				.try_select_first("h6")
				.ok()
				.and_then(|element| element.text())
		})
		.ok_or_else(|| error!("Text not found"))
}

pub trait KeyPage {
	fn key(&self) -> Result<String>;
}

impl KeyPage for Document {
	fn key(&self) -> Result<String> {
		let key = self
			.try_select("script:not([*])")?
			.find_map(|element| {
				let data = element.data()?;
				data.contains("var").then_some(data)
			})
			.ok_or_else(|| error!("No script content contains `var`"))?
			.split('\'')
			.nth(1)
			.ok_or_else(|| error!("Key not found"))?
			.into();
		Ok(key)
	}
}

pub trait ChapterPage {
	fn pages(&self) -> Result<Vec<Page>>;
	fn dnt(&self) -> Option<String>;
}

impl ChapterPage for Document {
	fn pages(&self) -> Result<Vec<Page>> {
		let key = self.key()?;
		let json = self
			.try_select("script:not([*])")?
			.find_map(|element| {
				let data = element.data()?;
				data.contains("var").then_some(data)
			})
			.ok_or_else(|| error!("No script content contains `var`"))?
			.split_once("var contentKey = '")
			.ok_or_else(|| error!("String not found: `var contentKey = '`"))?
			.1
			.split_once("';")
			.ok_or_else(|| error!("String not found: `';`"))?
			.0
			.decrypt(&key)?;
		serde_json::from_slice::<Vec<page_list::Item>>(&json)?
			.into_iter()
			.map(TryInto::try_into)
			.collect()
	}

	fn dnt(&self) -> Option<String> {
		self.select_first("#dnt")?.attr("value")
	}
}

trait TrySelect {
	fn try_select<S: AsRef<str>>(&self, css_query: S) -> Result<ElementList>;
	fn try_select_first<S: AsRef<str>>(&self, css_query: S) -> Result<Element>;
}

impl TrySelect for Document {
	fn try_select<S: AsRef<str>>(&self, css_query: S) -> Result<ElementList> {
		self.select(&css_query)
			.ok_or_else(|| error!("No element found for selector: `{}`", css_query.as_ref()))
	}

	fn try_select_first<S: AsRef<str>>(&self, css_query: S) -> Result<Element> {
		self.select_first(&css_query)
			.ok_or_else(|| error!("No element found for selector: `{}`", css_query.as_ref()))
	}
}
