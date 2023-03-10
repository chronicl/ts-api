use std::{collections::HashMap, fs::create_dir_all, fs::write, path::Path};
pub use ts_api_core::{ApiExtractor, ApiExtractorType, ApiHandler, ApiMethod, ApiRequest};
pub use ts_api_macros::api;

const TS_REQUEST: &str = include_str!("../ts/request.ts");
const TS_PROMISE: &str = include_str!("../ts/CancelablePromise.ts");

pub struct Api {
    server_url: String,
    pub router: poem::Route,
    // typescript file name -> typescript file content
    typescript: HashMap<String, String>,
    typescript_api: String,
}

impl Api {
    pub fn new(server_url: impl ToString) -> Self {
        Self {
            server_url: server_url.to_string(),
            router: poem::Route::new(),
            typescript: HashMap::new(),
            typescript_api: String::new(),
        }
    }

    pub fn register<T: ApiHandler + poem::Endpoint + 'static>(mut self, handler: T) -> Self {
        let file_name = T::ts_file_name();
        self.typescript
            .insert(file_name.clone(), T::typescript(&self.server_url));

        self.typescript_api.push_str(
            &format!("import {{ request as {file_name} }} from './{file_name}';\nexport {{ {file_name} }};\n\n", )
        );

        self.router = match T::METHOD {
            ApiMethod::Get => self.router.at(T::PATH, poem::get(handler)),
            ApiMethod::Post => self.router.at(T::PATH, poem::post(handler)),
            ApiMethod::Put => self.router.at(T::PATH, poem::put(handler)),
            ApiMethod::Delete => self.router.at(T::PATH, poem::delete(handler)),
            ApiMethod::Head => self.router.at(T::PATH, poem::head(handler)),
            ApiMethod::Options => self.router.at(T::PATH, poem::options(handler)),
            ApiMethod::Connect => self.router.at(T::PATH, poem::connect(handler)),
            ApiMethod::Patch => self.router.at(T::PATH, poem::patch(handler)),
            ApiMethod::Trace => self.router.at(T::PATH, poem::trace(handler)),
        };
        self
    }

    pub fn export_ts_client(&self, export_dir: impl AsRef<Path>) -> std::io::Result<()> {
        let api_dir = export_dir.as_ref().join("api");
        create_dir_all(&api_dir)?;

        write(export_dir.as_ref().join("request.ts"), TS_REQUEST)?;
        write(export_dir.as_ref().join("CancelablePromise.ts"), TS_PROMISE)?;
        write(api_dir.join("index.ts"), &self.typescript_api)?;
        for (file_name, content) in &self.typescript {
            write(api_dir.join(file_name).with_extension("ts"), content)?;
        }
        Ok(())
    }
}

#[test]
fn test_api() {
    use crate as ts_api;
    use poem::web::{Data, Json, Path, Query};
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;

    #[derive(TS, Deserialize, Serialize)]
    struct Auth {
        email: String,
        password: String,
    }

    #[derive(TS, Deserialize, Serialize)]
    struct AuthResponse {
        token: String,
    }

    #[derive(TS, Deserialize, Serialize)]
    #[ts(type = "enum")]
    enum Error {
        NotAndEmail,
        InvalidPassword,
    }

    #[api(method = "get", path = "/user")]
    async fn user(b: Json<String>) -> Json<u32> {
        Json(0)
    }

    #[api(method = "get", path = "/user/login")]
    async fn login(b: Json<Auth>) -> Json<Result<AuthResponse, Error>> {
        Json(Ok(AuthResponse { token: "".into() }))
    }

    let api = Api::new("http://localhost:3000")
        .register(user)
        .register(login);

    api.export_ts_client("test-api-ts").unwrap();
}
