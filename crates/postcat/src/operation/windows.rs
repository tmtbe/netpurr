use std::cell::RefCell;
use std::rc::Rc;

use crate::data::collections::{Collection, CollectionFolder};
use crate::data::http::HttpRecord;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct OpenWindows {
    pub save_opened: bool,
    pub edit: bool,
    pub collection_opened: bool,
    pub folder_opened: bool,
    pub cookies_opened: bool,
    pub http_record: HttpRecord,
    pub default_path: Option<String>,
    pub collection: Option<Collection>,
    pub parent_folder: Rc<RefCell<CollectionFolder>>,
    pub folder: Option<Rc<RefCell<CollectionFolder>>>,
    pub crt_id: String,
    pub save_crt_opened: bool,
}

impl OpenWindows {
    pub fn open_crt_save(&mut self, crt_id: String) {
        self.crt_id = crt_id;
        self.save_crt_opened = true;
    }
    pub fn open_save(&mut self, http_record: HttpRecord, default_path: Option<String>) {
        self.http_record = http_record;
        self.default_path = default_path;
        self.save_opened = true;
        self.edit = false;
    }
    pub fn open_edit(&mut self, http_record: HttpRecord, default_path: String) {
        self.http_record = http_record;
        self.default_path = Some(default_path);
        self.save_opened = true;
        self.edit = true
    }
    pub fn open_collection(&mut self, collection: Option<Collection>) {
        self.collection = collection;
        self.collection_opened = true;
    }
    pub fn open_folder(
        &mut self,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Option<Rc<RefCell<CollectionFolder>>>,
    ) {
        self.collection = Some(collection);
        self.parent_folder = parent_folder;
        self.folder = folder;
        self.folder_opened = true;
    }

    pub fn open_cookies(&mut self) {
        self.cookies_opened = true
    }
}
