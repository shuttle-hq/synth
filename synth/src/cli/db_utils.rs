use crate::cli::import::DataFormat;

pub struct DataSourceParams {
    pub uri: Option<String>, //perhaps uri is not a good name here as this could be a file path
    pub schema: Option<String>,
    pub data_format: DataFormat,
}
