use hyper::{Server, Request, Body, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use tokio::fs::read;
use async_process::Command;
use base64;

use std::error::Error;
use hyper::header::CONTENT_TYPE;

async fn get_desktop_picture() -> Result<String, Box<dyn Error + Send + Sync>> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Finder\" to get POSIX path of (get desktop picture as alias)")
        .output()
        .await?;
    let path  = String::from_utf8(output.stdout)?.trim().to_string();
    println!("{}", path);
    Ok(path)
}

async fn to_base64(file_path: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let data = read(file_path).await?;
    let encoded = base64::encode(data);
    Ok(encoded)
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/get_desktop_picture_base64" => {
            match get_desktop_picture().await {
                Ok(file_path) => {
                    match to_base64(file_path).await {
                        Ok(base64) => {
                            Ok(Response::builder()
                                .status(StatusCode::OK)
                                .header(CONTENT_TYPE, "text/plain")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(Body::from(base64))
                                .unwrap())
                        },
                        Err(_) => {
                            Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to read file"))
                                .unwrap())
                        }
                    }
                },
                Err(_) => {
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to get desktop picture"))
                        .unwrap())
                }
            }
        },
        "/get_desktop_picture" => {
            match get_desktop_picture().await {
                Ok(file_path) => {
                    match tokio::fs::read(&file_path).await {
                        Ok(data) => {
                            Ok(Response::builder()
                                .status(StatusCode::OK)
                                .header(CONTENT_TYPE, "image/jpg") // 根据你的桌面图片的真实格式进行设置，比如"image/png"
                                .header("Access-Control-Allow-Origin", "*")
                                .body(Body::from(data))
                                .unwrap())
                        },
                        Err(_) => {
                            Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to read file"))
                                .unwrap())
                        }
                    }
                },
                Err(_) => {
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to get desktop picture"))
                        .unwrap())
                }
            }
        },
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap())
        }
    }
}
#[tokio::main]
async fn main() {
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, hyper::Error>(service_fn(handle_request)) }
    });

    let server = Server::bind(&([127, 0, 0, 1], 3000).into())
        .serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}