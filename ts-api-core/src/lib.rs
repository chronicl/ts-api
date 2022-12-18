use convert_case::{Case, Casing};
use poem::web::{
    cookie::{Cookie, CookieJar},
    Data, Form, Json, Path, Query,
};
use ts_rs::{Dependencies, TS};

pub trait ApiExtractor {
    type Inner;
    const TYPE: Option<ApiExtractorType>;

    fn param() -> Option<String> {
        None
    }

    fn options() -> Option<Vec<String>> {
        None
    }

    fn response_type() -> Option<String> {
        None
    }

    fn add_dependencies(dependencies: &mut Dependencies) {}
}

// Todo: Should redo this to remove ApiExtractorType completely
// and just implement ApiExtractor for each type manually (or use a helper macro)
macro_rules! impl_api_extractor {
    () => {
        fn param() -> Option<String> {
            let mut param = match Self::TYPE? {
                ApiExtractorType::Json => "body",
                ApiExtractorType::Path => "path",
                ApiExtractorType::Query => "query",
            }
            .to_string();
            param.push_str(": ");
            param.push_str(&Self::Inner::name_with_generics());
            Some(param)
        }

        fn options() -> Option<Vec<String>> {
            Some(
                match Self::TYPE? {
                    ApiExtractorType::Json => {
                        vec!["body", "mediaType: 'application/json; charset=utf-8'"]
                    }
                    // Todo: Add support for Path simply being String, (String, String), ...
                    ApiExtractorType::Path => vec!["path"],
                    ApiExtractorType::Query => vec!["query"],
                }
                .into_iter()
                .map(str::to_string)
                .collect(),
            )
        }

        fn response_type() -> Option<String> {
            Some(format!(
                ": CancelablePromise<{}>",
                Self::Inner::name_with_generics()
            ))
        }

        fn add_dependencies(dependencies: &mut Dependencies) {
            dependencies.add::<Self::Inner>();
        }
    };
}

#[derive(Default)]
pub struct ApiRequest {
    params: Vec<String>,
    response_type: Option<String>,
    options: Vec<String>,
    types: Dependencies,
}

impl ApiRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_param<T: ApiExtractor>(&mut self) {
        if let Some(p) = T::param() {
            self.params.push(p);
            if let Some(options) = T::options() {
                self.options.extend(options);
            }
            T::add_dependencies(&mut self.types);
        }
    }

    pub fn register_response_type<T: ApiExtractor>(&mut self) {
        self.response_type = T::response_type();
        T::add_dependencies(&mut self.types);
    }

    pub fn finish(
        &mut self,
        server_url: impl AsRef<str>,
        method: impl AsRef<str>,
        path: impl AsRef<str>,
    ) -> String {
        let params: String = self.params.join(", ");
        let options: String = self.options.join(",\n\t\t\t");
        let response_type = self.response_type.to_owned().unwrap_or_default();
        let server_url = server_url.as_ref();
        let method = method.as_ref();
        let path = path.as_ref();

        let mut request = String::new();
        request.push_str("import { request as __request } from '../request';\n");
        request.push_str("import { CancelablePromise } from '../CancelablePromise';\n\n");

        for ty in self.types.values() {
            request.push_str(&format!("export {}\n\n", ty.ts_declaration))
        }

        let request_fn = format!(
            r#"export function request({params}){response_type} {{
    return __request(
        {{ url: '{server_url}' }},
        {{
            method: '{method}',
            url: '{path}',
            {options}
        }}
    );
}}"#
        );
        request.push_str(&request_fn);

        request
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApiExtractorType {
    Json,
    Path,
    Query,
}

impl<T: TS> ApiExtractor for Json<T> {
    type Inner = T;
    const TYPE: Option<ApiExtractorType> = Some(ApiExtractorType::Json);

    impl_api_extractor!();
}

impl<T: TS> ApiExtractor for Query<T> {
    type Inner = T;
    const TYPE: Option<ApiExtractorType> = Some(ApiExtractorType::Query);

    impl_api_extractor!();
}

// Todo: Add Form support
impl<T: TS> ApiExtractor for Form<T> {
    type Inner = T;
    // Should be Form once supported
    const TYPE: Option<ApiExtractorType> = None;
}

impl<T: TS> ApiExtractor for Path<T> {
    type Inner = T;
    const TYPE: Option<ApiExtractorType> = Some(ApiExtractorType::Path);

    impl_api_extractor!();
}

impl<T> ApiExtractor for Data<T> {
    type Inner = T;
    const TYPE: Option<ApiExtractorType> = None;
}

impl ApiExtractor for CookieJar {
    type Inner = ();
    const TYPE: Option<ApiExtractorType> = None;
}

impl<T: ApiExtractor> ApiExtractor for &T {
    type Inner = T::Inner;
    const TYPE: Option<ApiExtractorType> = T::TYPE;
}

pub enum TsType {
    Json,
    Path,
    Query,
}

pub trait ApiHandler {
    const METHOD: ApiMethod;
    const PATH: &'static str;

    fn typescript(server_url: impl AsRef<str>) -> String;

    fn ts_path() -> String {
        Self::PATH
            .split('/')
            .map(|s| {
                if let Some(s) = s.strip_prefix(':') {
                    format!("{{{s}}}")
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    // Todo: This should maybe include the method since it's possible to have the same path with different methods
    fn ts_file_name() -> String {
        let file_name = Self::PATH
            .split('/')
            .map(|s| {
                if let Some(s) = s.strip_prefix(':') {
                    s
                } else {
                    s
                }
            })
            .collect::<Vec<_>>()
            .join("_")
            .to_case(Case::Camel);
        if file_name.is_empty() {
            "root".to_string()
        } else {
            file_name
        }
    }
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

impl ApiMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Connect => "CONNECT",
            Self::Patch => "PATCH",
            Self::Trace => "TRACE",
        }
    }
}
