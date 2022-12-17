use poem::web::Json;
use ts_api::{api, ApiHandler};
use ts_rs::TS;

#[api(method = "get", path = "/")]
async fn a(_b: Json<String>) -> Json<u32> {
    Json(0)
}

fn main() {
    assert_eq!(<a as ApiHandler>::Request::name(), "string");
    assert_eq!(<a as ApiHandler>::Response::name(), "number");
}
