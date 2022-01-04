#[derive(clap::ArgEnum, Clone, Copy, Debug)]
pub enum Format {
    Human,
    Json,
}
