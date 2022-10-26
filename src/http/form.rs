use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use hyper::{header::CONTENT_TYPE, HeaderMap};
use multer::{Field, Multipart};
use multimap::MultiMap;
use tempfile::Builder;
use textnonce::TextNonce;
use tokio::{fs::File, io::AsyncWriteExt};

use super::errors::ParseError;

use crate::http::request::ReqBody;

#[derive(Debug)]
pub struct FormData {
    pub fields: MultiMap<String, String>,
    pub files: MultiMap<String, FilePart>,
}

impl FormData {
    pub fn new() -> FormData {
        FormData {
            fields: MultiMap::new(),
            files: MultiMap::new(),
        }
    }
    pub(crate) async fn read(headers: &HeaderMap, body: ReqBody) -> Result<FormData, ParseError> {
        match headers.get(CONTENT_TYPE) {
            Some(ctype) if ctype == "application/x-www-form-urlencoded" => {
                let data = hyper::body::to_bytes(body)
                    .await
                    .map(|d| d.to_vec())
                    .map_err(ParseError::Hyper)?;
                let mut form_data = FormData::new();
                form_data.fields = form_urlencoded::parse(&data).into_owned().collect();
                Ok(form_data)
            }
            Some(ctype) if ctype.to_str().unwrap_or("").starts_with("multipart/") => {
                let mut form_data = FormData::new();
                if let Some(boundary) = headers
                    .get(CONTENT_TYPE)
                    .and_then(|ct| ct.to_str().ok())
                    .and_then(|ct| multer::parse_boundary(ct).ok())
                {
                    let mut multipart = Multipart::new(body, boundary);
                    while let Some(mut field) = multipart.next_field().await? {
                        if let Some(name) = field.name().map(|s| s.to_owned()) {
                            if field.headers().get(CONTENT_TYPE).is_some() {
                                form_data
                                    .files
                                    .insert(name, FilePart::create(&mut field).await?);
                            } else {
                                form_data.fields.insert(name, field.text().await?);
                            }
                        }
                    }
                }
                Ok(form_data)
            }
            _ => Err(ParseError::InvalidContentType),
        }
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone, Debug)]
pub struct FilePart {
    name: Option<String>,
    /// The headers of the part
    headers: HeaderMap,
    /// A temporary file containing the file content
    path: PathBuf,
    /// Optionally, the size of the file.  This is filled when multiparts are parsed, but is
    /// not necessary when they are generated.
    size: Option<usize>,
    // The temporary directory the upload was put into, saved for the Drop trait
    temp_dir: Option<PathBuf>,
}

impl FilePart {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn name_mut(&mut self) -> Option<&mut String> {
        self.name.as_mut()
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn size(&self) -> Option<usize> {
        self.size
    }
    pub fn do_not_delete_on_drop(&mut self) {
        self.temp_dir = None;
    }
    pub async fn create(field: &mut Field<'_>) -> Result<FilePart, ParseError> {
        let mut path =
            tokio::task::spawn_blocking(|| Builder::new().prefix("salvo_http_multipart").tempdir())
                .await
                .expect("Runtime spawn blocking poll error")?
                .into_path();

        let temp_dir = Some(path.clone());
        let name = field.file_name().map(|s| s.to_owned());
        path.push(format!(
            "{}.{}",
            TextNonce::sized_urlsafe(32).unwrap().into_string(),
            name.as_deref()
                .and_then(|name| { Path::new(name).extension().and_then(OsStr::to_str) })
                .unwrap_or("unknown")
        ));
        let mut file = File::create(&path).await?;
        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
        }
        Ok(FilePart {
            name,
            headers: field.headers().to_owned(),
            path,
            size: None,
            temp_dir,
        })
    }
}

impl Drop for FilePart {
    fn drop(&mut self) {
        if let Some(temp_dir) = &self.temp_dir {
            let path = self.path.clone();
            let temp_dir = temp_dir.to_owned();
            tokio::task::spawn_blocking(move || {
                std::fs::remove_file(&path).ok();
                std::fs::remove_dir(temp_dir).ok();
            });
        }
    }
}