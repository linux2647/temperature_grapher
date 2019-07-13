use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Opt {
    /// Hour span
    #[structopt(
        short = "H",
        long = "hours",
        env = "HOURS",
        default_value = "72",
        help = "Number of hours of data to obtain"
    )]
    pub hour_span: u32,

    /// Database host
    #[structopt(short = "h", long = "host", env = "DB_HOST")]
    pub host: String,
    /// Database port
    #[structopt(short = "p", long = "port", env = "DB_PORT", default_value = "5432")]
    pub port: u16,
    /// Database username
    #[structopt(short = "U", long = "username", env = "DB_USER")]
    pub username: String,
    /// Database password
    #[structopt(short = "P", long = "password", env = "DB_PASS")]
    pub password: String,
    /// Database name
    #[structopt(short = "n", long = "name", env = "DB_NAME")]
    pub dbname: String,

    #[cfg(not(feature = "aws-lambda"))]
    /// File path to output to
    #[structopt(
        parse(from_os_str),
        short = "o",
        long = "output",
        env = "OUTPUT",
        default_value = "out.png"
    )]
    pub output: PathBuf,

    #[cfg(feature = "aws-lambda")]
    #[structopt(flatten)]
    pub lambda_opts: LambdaOpt,
}

#[cfg(feature = "aws-lambda")]
#[derive(Debug, StructOpt)]
pub struct LambdaOpt {
    /// S3 Bucket
    #[structopt(env = "S3_BUCKET")]
    pub bucket: String,
    /// S3 Key
    #[structopt(env = "S3_PATH")]
    pub key: PathBuf,
    /// Make files public
    #[structopt(env = "PUBLIC")]
    pub public: bool,
}

pub fn get_config() -> Opt {
    Opt::from_args()
}
