use chrono::{DateTime, Utc};
use http::Uri;

/// Structured data, deserialized from an API response.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Article {
    /// The name of the author.
    pub author: Option<String>,
    /// The main body content.
    #[serde(default)]
    pub content: String,
    /// The publication date.
    pub date_published: Option<DateTime<Utc>>,
    /// The dek.
    pub dek: Option<String>,
    /// The text direction of `content`.
    pub direction: TextDirection,
    /// A short description.
    #[serde(default)]
    pub excerpt: String,
    /// The url of the lead image (if present).
    #[serde(with = "serde_uri_opt")]
    pub lead_image_url: Option<Uri>,
    /// The url of the next page (if present).
    #[serde(with = "serde_uri_opt")]
    pub next_page_url: Option<Uri>,
    /// The number of pages included in `content`.
    #[serde(default = "default_page_field_value")]
    pub rendered_pages: u64,
    /// The title of the article.
    #[serde(default)]
    pub title: String,
    /// The total number of pages.
    #[serde(default = "default_page_field_value")]
    pub total_pages: u64,
    /// The original url.
    #[serde(with = "serde_uri")]
    pub url: Uri,
    /// The total number of words.
    #[serde(default)]
    pub word_count: u64,
    /// Private field for backwards compatibility.
    #[serde(default, skip)]
    _ext: (),
}

/// Represents the text direction of parsed body content.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    /// Left to right.
    Ltr,
    /// Right to left.
    Rtl,
}

impl TextDirection {
    /// Returns `true` if the direction is a [`Ltr`] value.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate mercury;
    /// #
    /// # use mercury::TextDirection;
    /// #
    /// # fn main() {
    /// let dir = TextDirection::Ltr;
    /// assert_eq!(dir.is_ltr(), true);
    ///
    /// let dir = TextDirection::Rtl;
    /// assert_eq!(dir.is_ltr(), false);
    /// # }
    /// ```
    ///
    /// [`Ltr`]: #variant.Ltr
    pub fn is_ltr(&self) -> bool {
        *self == TextDirection::Ltr
    }

    /// Returns `true` if the direction is a [`Rtl`] value.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate mercury;
    /// #
    /// # use mercury::TextDirection;
    /// #
    /// # fn main() {
    /// let dir = TextDirection::Ltr;
    /// assert_eq!(dir.is_rtl(), false);
    ///
    /// let dir = TextDirection::Rtl;
    /// assert_eq!(dir.is_rtl(), true);
    /// # }
    /// ```
    ///
    /// [`Rtl`]: #variant.Rtl
    pub fn is_rtl(&self) -> bool {
        *self == TextDirection::Rtl
    }
}

impl Default for TextDirection {
    fn default() -> TextDirection {
        TextDirection::Ltr
    }
}

/// Returns the default value for `rendered_pages` and `total_pages` if either
/// field is blank during deserialization.
fn default_page_field_value() -> u64 {
    1
}

/// Custom `deserialize` and `serialize` functions for `Uri` types.
mod serde_uri {
    use http::Uri;
    use serde::de::{self, Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    pub fn serialize<S>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!("{}", uri).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uri, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

/// Custom `deserialize` and `serialize` functions for optional `Uri` types.
mod serde_uri_opt {
    use http::Uri;
    use serde::de::{self, Deserialize, Deserializer};
    use serde::ser::Serializer;

    use super::serde_uri;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Uri>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<String>::deserialize(deserializer)? {
            Some(ref value) => value.parse().map(Some).map_err(de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize<S>(uri: &Option<Uri>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *uri {
            Some(ref value) => serde_uri::serialize(value, serializer),
            None => serializer.serialize_none(),
        }
    }
}
