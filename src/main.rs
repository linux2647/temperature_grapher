mod config;
#[cfg(not(feature = "aws-lambda"))]
use crate::config::get_config;

mod lib;

#[cfg(feature = "aws-lambda")]
mod lambda;

// Regular main
#[cfg(not(feature = "aws-lambda"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config();
    let conn = lib::DatabaseConnection {
        host: config.host,
        port: config.port,
        username: config.username,
        password: config.password,
        name: config.dbname,
    };

    lib::build_graph(
        conn,
        config.hour_span as i32,
        config.output.to_str().unwrap(),
    )?;

    Ok(())
}

// Lambda main
#[cfg(feature = "aws-lambda")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    lambda::lambda_main()
}
