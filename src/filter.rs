use crate::Tag;
use itertools::Itertools;

pub trait ToFilterExpr {
    /// Tag equals
    fn equals<T: ToString>(self, s: T) -> FilterExpr;

    /// Tag contains
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

/// Abstraction over search filter
pub struct Filter {
    exprs: Vec<FilterExpr>,
}

impl Filter {
    pub fn new() -> Self {
        Self { exprs: Vec::new() }
    }

    pub fn with(filter: FilterExpr) -> Self {
        Self {
            exprs: vec![filter],
        }
    }

    pub fn and(mut self, other: FilterExpr) -> Filter {
        self.exprs.push(other);
        self
    }

    pub fn and_not(mut self, other: FilterExpr) -> Self {
        self.exprs.push(FilterExpr::Not(Box::new(other)));
        self
    }

    pub fn to_query(&self) -> Option<String> {
        if self.exprs.is_empty() {
            return None;
        }

        let joined = self
            .exprs
            .iter()
            .map(|filter| filter.to_query())
            .join(" AND ");

        Some(format!("({})", escape(&joined)))
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\'', "\\\'")
}
