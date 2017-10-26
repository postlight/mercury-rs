use chrono::{DateTime, Utc};

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
    /// The url of the lead image.
    pub lead_image_url: Option<String>,
    /// The url of the next page.
    pub next_page_url: Option<String>,
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
    pub url: String,
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
