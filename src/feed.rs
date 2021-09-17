use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};
use serde_derive::Deserialize;
use std::str::FromStr;

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
    pub fn into_titled_entries<S: AsRef<str>>(self, title_filters: &[S], is_title_blacklist: bool) -> HashMap<String, Vec<Entry>> {
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

impl FromStr for Feed {
    type Err = serde_xml_rs::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_xml_rs::from_str(s)
    }
}
