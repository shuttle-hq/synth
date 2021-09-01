use ndarray::{ArrayView, Ix1};

use std::convert::Infallible;

/// Trait for functions that produce a value from an input [`Array`](ndarray::Array) of prescribed
/// shape.
///
/// The type parameter `D` should probably be a [`Dimension`](ndarray::Dimension) for implementations
/// to be useful.
pub trait Decoder<D> {
    type Err: std::error::Error + 'static;

    /// The type of values returned.
    type Value;

    /// Compute and return a [`Self::Value`](Decoder::Value) from the input `tensor`.
    ///
    /// Implementations are allowed to panic if `tensor.shape() != self.shape()`.
    fn decode(&self, tensor: ArrayView<f32, D>) -> Result<Self::Value, Self::Err>;

    /// The shape that is required of a valid input of this decoder.
    fn shape(&self) -> D;
}

impl<'d, D, Dm> Decoder<Dm> for &'d D
where
    D: Decoder<Dm>,
{
    type Err = D::Err;
    type Value = D::Value;

    fn decode(&self, tensor: ArrayView<f32, Dm>) -> Result<Self::Value, Self::Err> {
        <D as Decoder<Dm>>::decode(self, tensor)
    }

    fn shape(&self) -> Dm {
        <D as Decoder<Dm>>::shape(self)
    }
}

pub struct MaxIndexDecoder<S> {
    index: Vec<S>,
}

impl<S> MaxIndexDecoder<S> {
    /// # Panics
    ///
    /// If `index` is empty.
    pub fn from_vec(index: Vec<S>) -> Self {
        assert!(
            !index.is_empty(),
            "passed `index` to `from_values` must not be empty"
        );
        Self { index }
    }
}

impl<S> Decoder<Ix1> for MaxIndexDecoder<S>
where
    S: Clone,
{
    type Err = Infallible;
    type Value = Option<S>;

    fn decode(&self, tensor: ArrayView<f32, Ix1>) -> Result<Self::Value, Self::Err> {
        let (idx, by) = tensor
            .iter()
            .enumerate()
            .max_by(|(_, l), (_, r)| l.total_cmp(r))
            .unwrap();
        if *by > (1. / tensor.len() as f32) {
            let value = self.index.get(idx).unwrap().clone();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn shape(&self) -> Ix1 {
        Ix1(self.index.len())
    }
}

#[cfg(test)]
pub mod tests {
    use super::Decoder;
    use super::MaxIndexDecoder;

    use ndarray::{Array, Ix1};

    #[test]
    fn decoder_max_index() {
        let decoder = MaxIndexDecoder::from_vec((0..10).collect());

        for idx in 0..10 {
            let mut input = Array::zeros(Ix1(10));
            *input.get_mut(idx).unwrap() = 1.;
            let output = decoder.decode(input.view()).unwrap();
            assert_eq!(output, Some(idx));
        }
    }
}
