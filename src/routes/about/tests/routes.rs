extern crate dotenv;

use crate::cache::manager::RedisCacheManager;
use crate::cache::redis::create_service_cache;
use crate::cache::{Cache, MockCache};
use crate::config::{build_number, chain_info_request_timeout, version, webhook_token};
use crate::routes::about::models::{About, ChainAbout};
use crate::routes::safes::models::Implementation;
use crate::tests::main::{setup_rocket, setup_rocket_with_mock_cache};
use crate::utils::http_client::{HttpClient, MockHttpClient, Request, Response};
use core::time::Duration;
use mockall::predicate::eq;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use rocket::{Build, Rocket, Route};
use std::sync::Arc;

fn routes_for_test() -> impl Into<Vec<Route>> {
    routes![
        super::super::routes::backbone,
        super::super::routes::get_about,
        super::super::routes::get_chains_about,
        super::super::routes::redis,
        super::super::routes::get_master_copies,
    ]
}

#[rocket::async_test]
async fn get_chains_about() {
    let mock_http_client = {
        let mut mock_http_client = MockHttpClient::new();
        mock_http_client
            .expect_get()
            .times(1)
            .return_once(move |_| {
                Ok(Response {
                    status_code: 200,
                    body: String::from(crate::tests::json::CHAIN_INFO_RINKEBY),
                })
            });
        mock_http_client
    };
    let expected = ChainAbout {
        transaction_service_base_uri: "https://safe-transaction.rinkeby.staging.gnosisdev.com"
            .to_string(),
        about: About {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: version(),
            build_number: build_number(),
        },
    };

    let client = Client::tracked(setup_rocket(mock_http_client, routes_for_test()))
        .await
        .expect("valid rocket instance");
    let response = {
        let mut response = client.get("/v1/chains/4/about");
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().await.unwrap(),
        serde_json::to_string(&expected).unwrap()
    );
}

#[rocket::async_test]
async fn get_about() {
    let mock_http_client = {
        let mut mock_http_client = MockHttpClient::new();
        mock_http_client.expect_get().times(0);
        mock_http_client
    };
    let expected = About {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: version(),
        build_number: build_number(),
    };

    let client = Client::tracked(setup_rocket(mock_http_client, routes_for_test()))
        .await
        .expect("valid rocket instance");
    let response = {
        let mut response = client.get("/about");
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().await.unwrap(),
        serde_json::to_string(&expected).unwrap()
    );
}

#[rocket::async_test]
async fn get_master_copies() {
    let chain_request = {
        let mut chain_request = Request::new(config_uri!("/v1/chains/{}/", 137));
        chain_request.timeout(Duration::from_millis(chain_info_request_timeout()));
        chain_request
    };
    let mock_http_client = {
        let mut mock_http_client = MockHttpClient::new();
        mock_http_client
            .expect_get()
            .times(1)
            .with(eq(chain_request))
            .return_once(move |_| {
                Ok(Response {
                    status_code: 200,
                    body: String::from(crate::tests::json::CHAIN_INFO_POLYGON),
                })
            });
        mock_http_client
            .expect_get()
            .times(1)
            .with(eq(Request::new(
                "https://safe-transaction-polygon.staging.gnosisdev.com/api/v1/about/master-copies/"
                    .to_string(),
            )))
            .return_once(move |_| {
                Ok(Response {
                    status_code: 200,
                    body: String::from(crate::tests::json::POLYGON_MASTER_COPIES),
                })
            });
        mock_http_client
    };
    let expected = vec![
        Implementation {
            address: "0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552".to_string(),
            version: "1.3.0".to_string(),
        },
        Implementation {
            address: "0x3E5c63644E683549055b9Be8653de26E0B4CD36E".to_string(),
            version: "1.3.0+L2".to_string(),
        },
    ];

    let client = Client::tracked(setup_rocket(mock_http_client, routes_for_test()))
        .await
        .expect("valid rocket instance");
    let response = {
        let mut response = client.get("/v1/chains/137/about/master-copies");
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string().await.unwrap(),
        serde_json::to_string(&expected).unwrap()
    );
}

#[rocket::async_test]
async fn get_backbone() {
    let chain_request = {
        let mut chain_request = Request::new(config_uri!("/v1/chains/{}/", 137));
        chain_request.timeout(Duration::from_millis(chain_info_request_timeout()));
        chain_request
    };
    let mock_http_client = {
        let mut mock_http_client = MockHttpClient::new();
        mock_http_client
            .expect_get()
            .times(1)
            .with(eq(chain_request))
            .return_once(move |_| {
                Ok(Response {
                    status_code: 200,
                    body: String::from(crate::tests::json::CHAIN_INFO_POLYGON),
                })
            });
        mock_http_client
            .expect_get()
            .times(1)
            .with(eq(Request::new(
                "https://safe-transaction-polygon.staging.gnosisdev.com/api/v1/about/".to_string(),
            )))
            .return_once(move |_| {
                Ok(Response {
                    status_code: 200,
                    body: String::from("{\"json\":\"json\"}"),
                })
            });
        mock_http_client
    };
    let expected = "{\"json\":\"json\"}";

    let client = Client::tracked(setup_rocket(mock_http_client, routes_for_test()))
        .await
        .expect("valid rocket instance");
    let response = {
        let mut response = client.get("/v1/chains/137/about/backbone");
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().await.unwrap(), expected);
}

#[rocket::async_test]
async fn get_redis() {
    let mock_http_client = {
        let mut mock_http_client = MockHttpClient::new();
        mock_http_client.expect_get().times(0);
        mock_http_client
    };
    let mock_default_cache = {
        let mut mock_cache = MockCache::new();
        mock_cache
            .expect_info()
            .times(1)
            .return_once(move || Some(String::from("Cache info")));
        mock_cache
    };

    let mock_info_cache = MockCache::new();

    let client = Client::tracked(setup_rocket_with_mock_cache(
        mock_http_client,
        mock_default_cache,
        mock_info_cache,
        routes_for_test(),
    ))
    .await
    .expect("valid rocket instance");
    let response = {
        let mut response = client.get(format!("/about/redis/{}", webhook_token()));
        response.add_header(Header::new("Host", "test.gnosis.io"));
        response.dispatch().await
    };

    let expected = "Cache info";

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().await.unwrap(), expected);
}
