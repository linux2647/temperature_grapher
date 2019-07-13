use chrono::offset::Utc;
use chrono::DateTime;
use plotters::prelude::*;
use postgres::{Connection, TlsMode};

#[derive(Clone)]
pub struct PointMetadata {
    last_timestamp: DateTime<Utc>,
    time_units_ago: i32,
    internal_temperature: f32,
    external_temperature: f32,
}

#[allow(dead_code)]
impl PointMetadata {
    pub fn last_timestamp(&self) -> DateTime<Utc> {
        self.last_timestamp
    }
    pub fn internal_temperature(&self) -> f32 {
        self.internal_temperature
    }
    pub fn external_temperature(&self) -> f32 {
        self.external_temperature
    }
}

pub struct DatabaseConnection {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

impl std::string::ToString for DatabaseConnection {
    fn to_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name,
        )
    }
}

/**
 * Builds and writes a graph to disk (a PNG) for the last given set of hours
 *   (hour_span) to the given path (path), including both the internal and
 *   external temperatures.
 */
pub fn build_graph(
    conn: DatabaseConnection,
    hour_span: i32,
    path: &str,
) -> Result<PointMetadata, Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(path, (640, 480)).into_drawing_area();
    root.fill(&White)?;

    let (data, (min_temp, max_temp)) = get_data(conn, hour_span)?;

    // Add some margin to the data range
    let min_temp = min_temp - 5.;
    let max_temp = max_temp + 5.;

    let (from_date, to_date) = (-hour_span, 0);

    let mut chart = if cfg!(not(feature = "aws-lambda")) {
        ChartBuilder::on(&root)
            .margin(16)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(from_date..to_date, min_temp..max_temp)?
    } else {
        ChartBuilder::on(&root)
            .margin(16)
            // FIXME Running via AWS Lambda with text on the chart is currently
            // not supported. rusttype throws an "unknown font format" error.
            .build_ranged(from_date..to_date, min_temp..max_temp)?
    };

    // Draw grid lines
    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        // (x, y) points are (hours ago, internal temperature)
        data.clone()
            .into_iter()
            .map(|point| (point.time_units_ago, point.internal_temperature))
            .collect::<Vec<(i32, f32)>>(),
        &Blue,
    ))?;
    chart.draw_series(LineSeries::new(
        // (x, y) points are (hours ago, external temperature)
        data.clone()
            .into_iter()
            .map(|point| (point.time_units_ago, point.external_temperature))
            .collect::<Vec<(i32, f32)>>(),
        &Red,
    ))?;

    // FIXME having zero data points will panic
    Ok(data[data.len() - 1].clone())
}

/**
 * Fetch the data from the database.  Returns a set of data as well as the
 * combined minimum and maximum of internal and external temperatures.
 */
fn get_data(
    connection: DatabaseConnection,
    hour_span: i32,
) -> Result<(Vec<PointMetadata>, (f32, f32)), Box<dyn std::error::Error>> {
    let conn = Connection::connect(connection.to_string(), TlsMode::None).unwrap();
    let now = Utc::now();
    let mut min = std::f32::INFINITY;
    let mut max = std::f32::NEG_INFINITY;

    let query = conn.query(
        r#"select
                                 datetime, 
                                 internal,
                                 external
                              from temperature
                              where
                                 datetime > (now() - (format('%s hours', $1::int))::interval)
                              "#,
        &[&hour_span],
    )?;
    let data = query.iter().map(|row| {
        let date: DateTime<Utc> = row.get(0);
        let internal = row.get(1);
        let external = row.get(2);

        min = min.min(internal).min(external);
        max = max.max(internal).max(external);

        let range = now - date;
        let units = range.num_hours();

        PointMetadata {
            last_timestamp: date,
            time_units_ago: -units as i32,
            internal_temperature: internal,
            external_temperature: external,
        }
    });

    Ok((data.collect(), (min, max)))
}
