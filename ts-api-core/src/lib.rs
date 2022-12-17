pub trait ApiExtractor {
    type Inner;
    const TYPE: ApiExtractorType;
}

pub enum ApiExtractorType {
    None,
    Json,
    Query,
    Path,
}

use poem::web::{Data, Form, Json, Path, Query};

impl<T> ApiExtractor for Json<T> {
    type Inner = T;
    const TYPE: ApiExtractorType = ApiExtractorType::Json;
}

impl<T> ApiExtractor for Query<T> {
    type Inner = T;
    const TYPE: ApiExtractorType = ApiExtractorType::Query;
}

// Todo: Add Form support
impl<T> ApiExtractor for Form<T> {
    type Inner = T;
    // Should be Form once supported
    const TYPE: ApiExtractorType = ApiExtractorType::None;
}

impl<T> ApiExtractor for Path<T> {
    type Inner = T;
    const TYPE: ApiExtractorType = ApiExtractorType::Path;
}

impl<T> ApiExtractor for Data<T> {
    type Inner = T;
    const TYPE: ApiExtractorType = ApiExtractorType::None;
}

pub enum TsType {
    Json,
    Path,
    Query,
}

pub trait ApiHandler {
    const API: ApiRoute;
}

#[derive(Debug, Clone)]
pub struct ApiRoute {
    pub method: ApiMethod,
    pub path: &'static str,
}

#[derive(Debug, Copy, Clone, darling::FromMeta, Eq, PartialEq, Hash)]
#[darling(rename_all = "lowercase")]
pub enum ApiMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}
