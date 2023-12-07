use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use mime::Mime;
use rand::Rng;

const BOUNDARY_LEN: usize = 29;

#[macro_export]
macro_rules! extend {
    ($dst:expr, $($arg:tt)*) => {
        $dst.extend(format_args!($($arg)*).to_string().as_bytes().to_vec())
    };
}

fn opt_filename(path: &Path) -> Option<&str> {
    path.file_name().and_then(|filename| filename.to_str())
}

fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Uniform::from(0..=9))
        .take(len)
        .map(|num| num.to_string())
        .collect()
}

fn mime_filename(path: &Path) -> (Mime, Option<&str>) {
    let content_type = mime_guess::from_path(path);
    let filename = opt_filename(path);
    (content_type.first_or_octet_stream(), filename)
}

#[derive(Debug)]
pub struct MultipartBuilder {
    boundary: String,
    inner: Vec<u8>,
    data_written: bool,
}

impl Default for MultipartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl MultipartBuilder {
    /// New MultipartBuilder
    pub fn new() -> Self {
        Self {
            boundary: random_alphanumeric(BOUNDARY_LEN),
            inner: Vec::new(),
            data_written: false,
        }
    }

    /// Add text field
    pub fn add_text(mut self, name: &str, text: &str) -> Self {
        self.write_field_headers(name, None, None);
        self.inner.extend(text.as_bytes());
        self
    }

    /// Add file
    pub fn add_file<P: AsRef<Path>>(self, name: &str, path: P) -> Self {
        let path = path.as_ref();
        let (content_type, filename) = mime_filename(path);
        let mut file = File::open(path).expect(format!("open {:?} error", filename).as_str());
        self.add_stream(&mut file, name, filename, Some(content_type))
    }

    /// Add some stream
    pub fn add_stream<S: Read>(
        mut self,
        stream: &mut S,
        name: &str,
        filename: Option<&str>,
        content_type: Option<Mime>,
    ) -> Self {
        // This is necessary to make sure it is interpreted as a file on the server end.
        let content_type = Some(content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM));
        self.write_field_headers(name, filename, content_type);
        io::copy(stream, &mut self.inner).expect("add_stream io copy error");
        self
    }

    fn write_boundary(&mut self) {
        if self.data_written {
            self.inner.extend(b"\r\n");
        }

        extend!(
            self.inner,
            "-----------------------------{}\r\n",
            self.boundary
        )
    }

    fn write_field_headers(
        &mut self,
        name: &str,
        filename: Option<&str>,
        content_type: Option<Mime>,
    ) {
        self.write_boundary();
        if !self.data_written {
            self.data_written = true;
        }
        extend!(
            self.inner,
            "Content-Disposition: form-data; name=\"{name}\""
        );

        if let Some(filename) = filename {
            extend!(self.inner, "; filename=\"{filename}\"");
        }
        if let Some(content_type) = content_type {
            extend!(self.inner, "\r\nContent-Type: {content_type}");
        }
        self.inner.extend(b"\r\n\r\n")
    }

    /// Build multipart data
    pub fn build(mut self) -> (String, Vec<u8>) {
        if self.data_written {
            self.inner.extend(b"\r\n");
        }

        // always write the closing boundary, even for empty bodies
        extend!(
            self.inner,
            "-----------------------------{}--\r\n",
            self.boundary
        );
        (
            format!(
                "multipart/form-data; boundary=---------------------------{}",
                self.boundary
            ),
            self.inner,
        )
    }
}
