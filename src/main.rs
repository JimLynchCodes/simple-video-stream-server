use warp::Filter;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};
use bytes::Bytes;
use async_stream::stream;
use hyper::body::Body;
use std::convert::Infallible;

#[tokio::main]
async fn main() {
    // Set up the route to serve the video file at /video
    let video_route = warp::path("video")
        .and(warp::get())
        .and_then(stream_video);

    // Start the server
    warp::serve(video_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// Handler function to stream video
async fn stream_video() -> Result<impl warp::Reply, Infallible> {
    let path = "path/to/your/video.mp4";  // Change this to your video file path
    match File::open(path).await {
        Ok(file) => {
            // Create a byte stream from the file
            let stream = stream! {
                let mut file = file;
                let mut buffer = [0; 8192];  // 8 KB chunks
                loop {
                    let n = match file.read(&mut buffer).await {
                        Ok(n) if n == 0 => break, // End of file
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    yield Ok::<_, io::Error>(Bytes::copy_from_slice(&buffer[..n]));
                }
            };

            // Return the stream wrapped in hyper::Body with a Content-Type header
            let response = warp::reply::with_header(
                warp::reply::Response::new(Body::wrap_stream(stream)),
                "Content-Type",
                "video/mp4",
            );
            Ok(response)
        }
        Err(_) => {
            // Create a full HTTP response for the error case, setting status and headers directly
            let mut response = warp::http::Response::new(Body::from("Video file not found"));
            *response.status_mut() = warp::http::StatusCode::NOT_FOUND;
            response.headers_mut().insert("Content-Type", "text/plain".parse().unwrap());
            Ok(response)
        }
    }
}
