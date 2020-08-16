use config::Config;
use hyper::body::Body;
use reqwest::Client;
use std::{collections::HashMap, env};
use warp::{self, http::Response, Filter, Reply};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port: u16;
    if args.len() > 1 {
        port = args[1].parse::<u16>().unwrap();
    } else {
        println!("error: no port argument provided");
        return;
    }

    let endpoint = warp::path!("oauth" / "token")
        .and(warp::post())
        .and(warp::header("Accept"))
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(exchange_code)
        .map(|res| res);

    let addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        port,
    );

    warp::serve(endpoint).run(addr).await;
}

fn error_response(
    msg: &str,
    err: Option<String>,
    code: warp::http::StatusCode,
) -> Result<Response<Body>, warp::Rejection> {
    let body;
    if let Some(err) = err {
        body = format!("{{\"message\": \"{}\", \"error\":\"{}\"}}", msg, err);
    } else {
        body = format!("{{\"message\": \"{}\"}}", msg);
    }

    let res = Response::builder()
        .status(code)
        .header("Content-Type", "application/json")
        .body(body)
        .into_response();

    Ok(res)
}

fn bad_request(msg: &str) -> Result<Response<Body>, warp::Rejection> {
    error_response(msg, None, warp::http::StatusCode::BAD_REQUEST)
}

fn internal_server_error(msg: &str, err: String) -> Result<Response<Body>, warp::Rejection> {
    error_response(
        msg,
        Some(err),
        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
    )
}

async fn exchange_code(
    accept: String,
    mut json: HashMap<String, String>,
) -> Result<Response<Body>, warp::Rejection> {
    if accept != "application/json" {
        return bad_request("Unsupported response content type");
    }

    let mut settings = Config::default();
    let settings = match settings.merge(config::File::with_name("Settings")) {
        Ok(s) => s,
        Err(err) => {
            return internal_server_error("Could not load proxy settings", format!("{}", err))
        }
    };
    let client_secret = match settings.get_str("client_secret") {
        Ok(secret) => secret,
        Err(err) => {
            return internal_server_error("Could not retrieve client secret", format!("{}", err))
        }
    };
    let token_url = match settings.get_str("token_url") {
        Ok(url) => url,
        Err(err) => {
            return internal_server_error("Could not retrieve token URL", format!("{}", err))
        }
    };

    json.insert("client_secret".to_string(), client_secret);

    let client = Client::new();
    let res = client
        .post(&token_url)
        .header("Accept", accept)
        .json(&json)
        .send()
        .await;

    match res {
        Ok(res) => {
            // Copy response data
            let status = res.status();
            let mut resp = Response::builder().status(&status);
            for (key, value) in res.headers().into_iter() {
                let value = match value.to_str() {
                    Ok(val) => val,
                    Err(err) => {
                        return internal_server_error(
                            "Error retrieving a token response header",
                            format!("{}", err),
                        )
                    }
                };
                resp = resp.header(key.as_str(), value);
            }
            let body = match res.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => {
                    return internal_server_error(
                        "Could not unwrap body of token response",
                        format!("{}", err),
                    )
                }
            };

            let response = resp.body(body);
            match response {
                Ok(response) => {
                    return Ok(warp::reply::with_status(response, status).into_response());
                }
                Err(err) => {
                    return internal_server_error(
                        "Could not unwrap constructed response",
                        format!("{}", err),
                    );
                }
            }
        }
        Err(err) => {
            return internal_server_error(
                "Error retrieving response from token URL",
                format!("{}", err),
            );
        }
    }
}
