use arrow::error::ArrowError;
use ndarray::ShapeError;

#[derive(Debug)]
pub enum Error {
    Arrow(ArrowError),
    Shape(ShapeError),
    Encoder(Box<dyn std::error::Error + 'static>),
    Model(Box<dyn std::error::Error + 'static>),
    Decoder(Box<dyn std::error::Error + 'static>),
    Implementation(String),
}

impl Error {
    pub fn encoder<E: std::error::Error + 'static>(err: E) -> Self {
        Self::Encoder(Box::new(err))
    }

    pub fn model<E: std::error::Error + 'static>(err: E) -> Self {
        Self::Model(Box::new(err))
    }

    pub fn decoder<E: std::error::Error + 'static>(err: E) -> Self {
        Self::Decoder(Box::new(err))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arrow(arrow) => write!(f, "arrow error: {}", arrow),
            Self::Shape(shape) => write!(f, "shape error: {}", shape),
            Self::Encoder(encoder) => write!(f, "encoder error: {}", encoder),
            Self::Model(model) => write!(f, "model error: {}", model),
            Self::Decoder(decoder) => write!(f, "decoder error: {}", decoder),
            Self::Implementation(msg) => write!(f, "implementation error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<ArrowError> for Error {
    fn from(arrow: ArrowError) -> Self {
        Self::Arrow(arrow)
    }
}

impl From<ShapeError> for Error {
    fn from(shape: ShapeError) -> Self {
        Self::Shape(shape)
    }
}
