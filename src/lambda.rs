use chrono::offset::Utc;
use lambda_runtime::{error::HandlerError, lambda};
use log;
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use serde_derive::{Deserialize, Serialize};
use simple_logger;
use std::fs::File;
use std::io::prelude::*;

use crate::{config::get_config, lib};

// Event definition is empty because we don't use any input
#[derive(Serialize, Deserialize, Clone)]
struct CustomEvent {}

pub fn lambda_main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    // Check config
    let _ = get_config();
    lambda!(handler);

    Ok(())
}

// Lambda handler
#[cfg(feature = "aws-lambda")]
fn handler(e: CustomEvent, _c: lambda_runtime::Context) -> Result<CustomEvent, HandlerError> {
    let config = get_config();
    let conn = lib::DatabaseConnection {
        host: config.host,
        port: config.port,
        username: config.username,
        password: config.password,
        name: config.dbname,
    };
    let publish_publicly = if !config.lambda_opts.public {
        &"private"
    } else {
        &"public-read"
    };
    let client = S3Client::new(Region::default());

    // TODO instead of saving the file to disk, keep it in memory.
    // The newer version of plotters should allow this
    let temp_file = "/tmp/out.png";
    let metadata = lib::build_graph(conn, config.hour_span as i32, temp_file).unwrap();

    let mut f = File::open(temp_file).unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();

    // Save the graph to S3, with public read access
    client
        .put_object(PutObjectRequest {
            bucket: config.lambda_opts.bucket.clone(),
            key: config.lambda_opts.key.clone().to_str().unwrap().to_string(),
            body: Some(ByteStream::from(buffer)),
            acl: Some(publish_publicly.to_string()),
            content_type: Some("image/png".to_string()),
            ..Default::default()
        })
        .sync()
        .unwrap();

    let now = Utc::now().to_rfc3339();
    // TODO use a template, but this is quick and dirty
    let index = format!(r#"<!DOCTYPE html>
    <html>
        <head>
            <meta http-equiv="refresh" content="60" />
            <meta name="viewport" content="width=640, initial-scale=0.57">
            <title>Graph</title>
        </head>
        <body>
            <p><img src="{}" />
            <p>Last update: <span class="to-localtime">{}</span>
            <p>Internal temp: {}°C
            <p>External temp: {}°C
            <p>This page updated on: <span class="to-localtime">{}</span>
            <script src="https://cdn.jsdelivr.net/npm/luxon@1.16.1/build/global/luxon.min.js" integrity="sha256-wPBIBYP6MqgpbBtRCz57ItYjHk8BCLjehEcw4UZ2WwA=" crossorigin="anonymous"></script>
            <script>
                (function() {{
                    const loaded_on = document.createElement('p')
                    loaded_on.innerText = "This page last refreshed on: "

                    const timestamp = document.createElement('span')
                    timestamp.classList.add("to-localtime")
                    timestamp.innerText = new Date().toISOString()
                    loaded_on.append(timestamp)

                    document.querySelector('body').append(loaded_on)

                    const DateTime = luxon.DateTime
                    document.querySelectorAll('.to-localtime').forEach(el => {{
                        el.innerText = DateTime.fromISO(el.innerText)
                            .setLocale('en-GB')
                            .toLocaleString(DateTime.DATETIME_MED)
                    }})
                }})()
            </script>
        </body>
    </html>
    "#,
        config.lambda_opts.key.to_str().unwrap(),
        metadata.last_timestamp().to_rfc3339(),
        metadata.internal_temperature(),
        metadata.external_temperature(),
        now,
    );

    client
        .put_object(PutObjectRequest {
            bucket: config.lambda_opts.bucket,
            key: "index.html".to_string(),
            body: Some(ByteStream::from(index.as_bytes().to_owned())),
            acl: Some(publish_publicly.to_string()),
            content_type: Some("text/html; charset=utf-8".to_string()),
            ..Default::default()
        })
        .sync()
        .unwrap();

    Ok(e)
}
