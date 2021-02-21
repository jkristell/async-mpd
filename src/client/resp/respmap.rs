use multimap::MultiMap;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub struct RespMap {
    pub(crate) inner: MultiMap<String, String>,
}

impl RespMap {
    pub fn new() -> Self {
        Self {
            inner: MultiMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn from_string(input: String) -> Self {
        let mut map = MultiMap::new();

        for line in input.lines() {
            if let Some((k, v)) = line.split_once(": ") {
                log::info!("kv: {} {}", k, v);
                map.insert(k.into(), v.into());
            }
        }

        RespMap { inner: map }
    }

    pub fn from_iterator<'a>(input: impl Iterator<Item = &'a str>) -> Self {
        let mut map = MultiMap::new();

        for line in input {
            if let Some((k, v)) = line.split_once(": ") {
                log::info!("kv: {} {}", k, v);
                map.insert(k.into(), v.into());
            }
        }

        RespMap { inner: map }
    }

    pub fn insert(&mut self, key: &str, val: &str) {
        self.inner.insert(key.into(), val.into());
    }

    pub fn get<T: FromStr>(&mut self, key: &str) -> Option<T> {
        self.inner
            .remove(key)
            .and_then(|mut v| v.pop())
            .and_then(|v| v.parse().ok())
    }

    pub fn get_vec(&mut self, key: &str) -> Vec<String> {
        self.inner.remove(key).unwrap_or_default()
    }

    pub fn get_def<T: Default + FromStr>(&mut self, key: &str) -> T {
        self.get(key).unwrap_or_default()
    }

    pub fn as_bool(&mut self, key: &str) -> bool {
        self.get_def::<i32>(key) != 0
    }

    pub fn as_duration(&mut self, key: &str) -> Option<Duration> {
        let secs: f64 = self.get(key)?;
        Some(Duration::from_secs_f64(secs))
    }

    pub fn as_duration_def(&mut self, key: &str) -> Duration {
        self.as_duration(key).unwrap_or_default()
    }
}
