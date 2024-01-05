#![allow(unused_imports)]
#![deny(warnings, missing_debug_implementations, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Example
//! The following example demonstrates loading a [`cookie_store::CookieStore`] (re-exported in this crate) from disk, and using it within a
//! [`CookieStoreMutex`]. It then makes a series of requests, examining and modifying the contents
//! of the underlying [`cookie_store::CookieStore`] in between.
//! ```no_run
//! # tokio_test::block_on(async {
//! // Load an existing set of cookies, serialized as json
//! let cookie_store = {
//!   if let Ok(file) = std::fs::File::open("cookies.json")
//!     .map(std::io::BufReader::new)
//!     {
//!       // use re-exported version of `CookieStore` for crate compatibility
//!       reqwest_cookie_store::CookieStore::load_json(file).unwrap()
//!     }
//!     else
//!     {
//!       reqwest_cookie_store::CookieStore::new(None)
//!     }
//! };
//! let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
//! let cookie_store = std::sync::Arc::new(cookie_store);
//! {
//!   // Examine initial contents
//!   println!("initial load");
//!   let store = cookie_store.lock().unwrap();
//!   for c in store.iter_any() {
//!     println!("{:?}", c);
//!   }
//! }
//!
//! // Build a `reqwest` Client, providing the deserialized store
//! let client = reqwest::Client::builder()
//!     .cookie_provider(std::sync::Arc::clone(&cookie_store))
//!     .build()
//!     .unwrap();
//!
//! // Make a sample request
//! client.get("https://google.com").send().await.unwrap();
//! {
//!   // Examine the contents of the store.
//!   println!("after google.com GET");
//!   let store = cookie_store.lock().unwrap();
//!   for c in store.iter_any() {
//!     println!("{:?}", c);
//!   }
//! }
//!
//! // Make another request from another domain
//! println!("GET from msn");
//! client.get("https://msn.com").send().await.unwrap();
//! {
//!   // Examine the contents of the store.
//!   println!("after msn.com GET");
//!   let mut store = cookie_store.lock().unwrap();
//!   for c in store.iter_any() {
//!     println!("{:?}", c);
//!   }
//!   // Clear the store, and examine again
//!   store.clear();
//!   println!("after clear");
//!   for c in store.iter_any() {
//!     println!("{:?}", c);
//!   }
//! }
//!
//! // Get some new cookies
//! client.get("https://google.com").send().await.unwrap();
//! {
//!   // Write store back to disk
//!   let mut writer = std::fs::File::create("cookies2.json")
//!       .map(std::io::BufWriter::new)
//!       .unwrap();
//!   let store = cookie_store.lock().unwrap();
//!   store.save_json(&mut writer).unwrap();
//! }
//! # });
//!```

use std::sync::{
    LockResult, Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use bytes::Bytes;
use reqwest::header::HeaderValue;
use url;

pub use cookie_store::{CookieStore, RawCookie, RawCookieParseError};

fn set_cookies(
    cookie_store: &mut CookieStore,
    cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
    url: &url::Url,
) {
    let cookies = cookie_headers.filter_map(|val| {
        std::str::from_utf8(val.as_bytes())
            .map_err(RawCookieParseError::from)
            .and_then(RawCookie::parse)
            .map(|c| c.into_owned())
            .ok()
    });
    cookie_store.store_response_cookies(cookies, url);
}

fn cookies(cookie_store: &CookieStore, url: &url::Url) -> Option<HeaderValue> {
    let s = cookie_store
        .get_request_values(url)
        .map(|(name, value)| format!("{}={}", name, value))
        .collect::<Vec<_>>()
        .join("; ");

    if s.is_empty() {
        return None;
    }

    HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
}

/// A [`cookie_store::CookieStore`] wrapped internally by a [`std::sync::Mutex`], suitable for use in
/// async/concurrent contexts.
#[derive(Debug)]
pub struct CookieStoreMutex(Mutex<CookieStore>);

impl Default for CookieStoreMutex {
    /// Create a new, empty [`CookieStoreMutex`]
    fn default() -> Self {
        CookieStoreMutex::new(CookieStore::default())
    }
}

impl CookieStoreMutex {
    /// Create a new [`CookieStoreMutex`] from an existing [`cookie_store::CookieStore`].
    pub fn new(cookie_store: CookieStore) -> CookieStoreMutex {
        CookieStoreMutex(Mutex::new(cookie_store))
    }

    /// Lock and get a handle to the contained [`cookie_store::CookieStore`].
    pub fn lock(
        &self,
    ) -> Result<MutexGuard<'_, CookieStore>, PoisonError<MutexGuard<'_, CookieStore>>> {
        self.0.lock()
    }

    /// Consumes this [`CookieStoreMutex`], returning the underlying [`cookie_store::CookieStore`]
    pub fn into_inner(self) -> LockResult<CookieStore> {
        self.0.into_inner()
    }
}

impl reqwest::cookie::CookieStore for CookieStoreMutex {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let mut store = self.0.lock().unwrap();
        set_cookies(&mut store, cookie_headers, url);
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let store = self.0.lock().unwrap();
        cookies(&store, url)
    }
}

/// A [`cookie_store::CookieStore`] wrapped internally by a [`std::sync::RwLock`], suitable for use in
/// async/concurrent contexts.
#[derive(Debug)]
pub struct CookieStoreRwLock(RwLock<CookieStore>);

impl Default for CookieStoreRwLock {
    /// Create a new, empty [`CookieStoreRwLock`].
    fn default() -> Self {
        CookieStoreRwLock::new(CookieStore::default())
    }
}

impl CookieStoreRwLock {
    /// Create a new [`CookieStoreRwLock`] from an existing [`cookie_store::CookieStore`].
    pub fn new(cookie_store: CookieStore) -> CookieStoreRwLock {
        CookieStoreRwLock(RwLock::new(cookie_store))
    }

    /// Lock and get a read (non-exclusive) handle to the contained [`cookie_store::CookieStore`].
    pub fn read(
        &self,
    ) -> Result<RwLockReadGuard<'_, CookieStore>, PoisonError<RwLockReadGuard<'_, CookieStore>>>
    {
        self.0.read()
    }

    /// Lock and get a write (exclusive) handle to the contained [`cookie_store::CookieStore`].
    pub fn write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, CookieStore>, PoisonError<RwLockWriteGuard<'_, CookieStore>>>
    {
        self.0.write()
    }

    /// Consume this [`CookieStoreRwLock`], returning the underlying [`cookie_store::CookieStore`]
    pub fn into_inner(self) -> LockResult<CookieStore> {
        self.0.into_inner()
    }
}

impl reqwest::cookie::CookieStore for CookieStoreRwLock {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let mut write = self.0.write().unwrap();
        set_cookies(&mut write, cookie_headers, url);
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let read = self.0.read().unwrap();
        cookies(&read, url)
    }
}
