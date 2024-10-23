use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub struct EventPayload {
    #[serde(rename = "Event")]
    pub event: EventDetails,
    #[serde(rename = "Type")]
    pub event_type: EventType,
}
#[derive(Debug, Deserialize, Serialize)]
pub enum EventType {
    #[serde(rename = "pre-create")]
    PreCreate,
    #[serde(rename = "post-receive")]
    PostReceive,
    #[serde(rename = "post-create")]
    PostCreate,
    #[serde(rename = "pre-finish")]
    PreFinish,
    #[serde(rename = "post-finish")]
    PostFinish,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventDetails {
    #[serde(rename = "HTTPRequest")]
    pub http_request: HttpRequest,
    #[serde(rename = "Upload")]
    pub upload: Upload,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRequest {
    #[serde(rename = "Header")]
    pub header: Header,
    #[serde(rename = "Method")]
    pub method: String,
    #[serde(rename = "RemoteAddr")]
    pub remote_addr: String,
    #[serde(rename = "URI")]
    pub uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Header {
    #[serde(rename = "Accept")]
    pub accept: Option<Vec<String>>,
    #[serde(rename = "Accept-Encoding")]
    pub accept_encoding: Option<Vec<String>>,
    #[serde(rename = "Accept-Language")]
    pub accept_language: Option<Vec<String>>,
    #[serde(rename = "Cache-Control")]
    pub cache_control: Option<Vec<String>>,
    #[serde(rename = "Content-Length")]
    pub content_length: Option<Vec<String>>,
    #[serde(rename = "Content-Type")]
    pub content_type: Option<Vec<String>>,
    #[serde(rename = "Dnt")]
    pub dnt: Option<Vec<String>>,
    #[serde(rename = "Host")]
    pub host: Option<Vec<String>>,
    #[serde(rename = "Origin")]
    pub origin: Option<Vec<String>>,
    #[serde(rename = "Pragma")]
    pub pragma: Option<Vec<String>>,
    #[serde(rename = "Referer")]
    pub referer: Option<Vec<String>>,
    #[serde(rename = "Tus-Resumable")]
    pub tus_resumable: Option<Vec<String>>,
    #[serde(rename = "Upload-Offset")]
    pub upload_offset: Option<Vec<String>>,
    #[serde(rename = "User-Agent")]
    pub user_agent: Option<Vec<String>>,
    #[serde(rename = "X-Forwarded-For")]
    pub x_forwarded_for: Option<Vec<String>>,
    #[serde(rename = "X-Forwarded-Host")]
    pub x_forwarded_host: Option<Vec<String>>,
    #[serde(rename = "X-Forwarded-Port")]
    pub x_forwarded_port: Option<Vec<String>>,
    #[serde(rename = "X-Forwarded-Proto")]
    pub x_forwarded_proto: Option<Vec<String>>,
    #[serde(rename = "X-Forwarded-Server")]
    pub x_forwarded_server: Option<Vec<String>>,
    #[serde(rename = "X-Real-Ip")]
    pub x_real_ip: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "IsFinal")]
    pub is_final: bool,
    #[serde(rename = "IsPartial")]
    pub is_partial: bool,
    #[serde(rename = "MetaData")]
    pub metadata: MetaData,
    #[serde(rename = "Offset")]
    pub offset: u64,
    #[serde(rename = "PartialUploads")]
    pub partial_uploads: Option<String>,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "SizeIsDeferred")]
    pub size_is_deferred: bool,
    #[serde(rename = "Storage")]
    pub storage: Option<Storage>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MetaData {
    #[serde(rename = "filename")]
    pub filename: String,
    #[serde(rename = "filetype")]
    pub filetype: String,
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "relativePath")]
    pub relative_path: Option<String>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Storage {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Type")]
    pub storage_type: String,
}

impl Default for MetaData {
    fn default() -> Self {
        Self {
            filename: String::new(),
            filetype: String::new(),
            name: None,
            relative_path: None,
            file_type: None,
        }
    }
}

impl Default for Upload {
    fn default() -> Self {
        Self {
            id: String::new(),
            is_final: false,
            is_partial: false,
            metadata: MetaData::default(),
            offset: 0,
            partial_uploads: None,
            size: 0,
            size_is_deferred: false,
            storage: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangeFileInfo {
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PreCreateResponse {
    #[serde(rename = "ChangeFileInfo")]
    pub change_file_info: Option<ChangeFileInfo>,
    pub status: String,
}
