#[derive(clap::ArgEnum, Clone, Debug)]
pub enum Format {
    Human,
    Json,
}
