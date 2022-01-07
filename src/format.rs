#[derive(clap::ArgEnum, Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Human,
    Json,
}
