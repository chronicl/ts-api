pub use ts_api_core::{ApiHandler, ApiMethod, ApiRoute};
pub use ts_api_macros::api;
pub use ts_rs::TS;

pub struct Api {
    server_url: String,
    routes: Vec<ApiRoute>,
    pub router: poem::Route,
}

impl Api {
    pub fn new(server_url: impl ToString) -> Self {
        Self {
            server_url: server_url.to_string(),
            routes: vec![],
            router: poem::Route::new(),
        }
    }

    pub fn register<T: ApiHandler + poem::Endpoint + 'static>(mut self, handler: T) -> Self {
        self.routes.push(T::API);

        self.router = match T::API.method {
            ApiMethod::Get => self.router.at(T::API.path, poem::get(handler)),
            ApiMethod::Post => self.router.at(T::API.path, poem::post(handler)),
            ApiMethod::Put => self.router.at(T::API.path, poem::put(handler)),
            ApiMethod::Delete => self.router.at(T::API.path, poem::delete(handler)),
            ApiMethod::Head => self.router.at(T::API.path, poem::head(handler)),
            ApiMethod::Options => self.router.at(T::API.path, poem::options(handler)),
            ApiMethod::Connect => self.router.at(T::API.path, poem::connect(handler)),
            ApiMethod::Patch => self.router.at(T::API.path, poem::patch(handler)),
            ApiMethod::Trace => self.router.at(T::API.path, poem::trace(handler)),
        };
        self
    }

    pub fn ts_client(&self) -> String {
        let client = String::new();

        client
    }
}
