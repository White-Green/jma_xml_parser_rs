
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub lang: String,
    pub title: String,
    pub subtitle: String,
    pub updated: DateTime<Local>,
    pub id: String,
    pub link: Vec<Link>,
    pub rights: Rights,
    #[serde(rename = "entry")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub title: String,
    pub id: String,
    pub updated: DateTime<Local>,
    pub author: Author,
    pub link: Link,
    pub content: Content,
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub rel: Option<String>,
    pub href: String,
}

#[derive(Debug, Deserialize)]
pub struct Rights {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "$value")]
    pub item: String,
}

impl Feed {
    pub fn into_titled_entries(self, title_filters: &[impl AsRef<str>], is_title_blacklist: bool) -> HashMap<String, Vec<Entry>> {
        let title_filters = title_filters.iter().map(AsRef::as_ref).collect::<HashSet<_>>();
        let mut result: HashMap<_, Vec<_>> = HashMap::new();
        for entry in self.entries {
            if title_filters.contains(entry.title.as_str()) ^ is_title_blacklist {
                result.entry(entry.title.clone()).or_default().push(entry);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::feed::Feed;

    #[test]
    fn test_parse_feed() {
        let result: Feed = serde_xml_rs::from_str(include_str!("../regular_l.xml")).unwrap();
        let mut map = result.into_titled_entries(&["府県天気予報（Ｒ１）"], false);
        let list = map.get_mut("府県天気予報（Ｒ１）").unwrap();
        list.retain(|entry| entry.content.value == "【福岡県府県天気予報】");
        let list = map.get("府県天気予報（Ｒ１）").unwrap();
        dbg!(list.len());
        dbg!(list);
        for (i, entry) in list.iter().enumerate() {
            println!("curl {} -o {}.xml", entry.link.href, i);
        }
    }
}
