use itertools::Itertools;

#[derive(Copy, Clone, Debug)]
/// Track tags
pub enum Tag {
    Artist,
    ArtistSort,
    Album,
    AlbumSort,
    AlbumArtist,
    AlbumSortOrder,
    Title,
    Track,
    Name,
    Genre,
    Date,
    Composer,
    Performer,
    Conductor,
    Work,
    Grouping,
    Comment,
    Disc,
    Label,
    MusicbrainzArtistId,
    MusicbrainzAlbumId,
    MusicbrainzAlbumArtistId,
    MusicbrainzTrackId,
    MusicbrainzReleaseTrackId,
    MusicbrainzWorkId,
    Any,
}

pub trait ToFilterExpr {
    fn equals<T: ToString>(self, s: T) -> FilterExpr;

    fn contains<T: ToString>(self, s: T) -> FilterExpr;
}

impl ToFilterExpr for Tag {
    fn equals<T: ToString>(self, s: T) -> FilterExpr {
        FilterExpr::Equals(self, s.to_string())
    }

    fn contains<T: ToString>(self, s: T) -> FilterExpr {
        FilterExpr::Contains(self, s.to_string())
    }
}

/// Search expression used by search function
pub enum FilterExpr {
    Equals(Tag, String),
    Contains(Tag, String),
    Not(Box<FilterExpr>),

}

impl FilterExpr {
    pub fn to_query(&self) -> String {
        match self {
            FilterExpr::Equals(tag, s) => format!("({:?} == \"{}\")", tag, s),
            FilterExpr::Contains(tag, s) => format!("({:?} contains \"{}\")", tag, s),
            FilterExpr::Not(exp) => format!("!{}", exp.to_query()),
        }
    }
}

pub struct Filter {
    filters: Vec<FilterExpr>,
}

impl Filter {

    pub fn new() -> Self {
        Self {
            filters: Vec::new()
        }
    }

    pub fn with(filter: FilterExpr) -> Self {
        Self {
            filters: vec![filter]
        }
    }

    pub fn and(mut self, other: FilterExpr) -> Filter {
        self.filters.push(other);
        self
    }

    pub fn and_not(mut self, other: FilterExpr) -> Self {
        self.filters.push(FilterExpr::Not(Box::new(other)));
        self
    }

    pub fn to_query(&self) -> Option<String> {

        if self.filters.is_empty() {
            return None;
        }

        let joined = self.filters.iter()
            .map(|filter| filter.to_query())
            .join(" AND ");

        Some(format!("({})", escape(&joined)))
    }

}

fn escape(s: &str) -> String {
    s
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\'', "\\\'")
}