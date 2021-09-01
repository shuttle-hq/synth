use arrow::array::{GenericStringArray, StringOffsetSizeTrait};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;

use ndarray::{ArrayViewMut, Axis, Ix1, Slice};

use std::collections::HashSet;
use std::convert::Infallible;

/// Trait for functions that compute an [`Array`](ndarray::Array) of prescribed shape from an input
/// [`RecordBatch`](arrow::record_batch::RecordBatch).
///
/// The type parameter `D` should probably be a [`Dimension`](ndarray::Dimension) for
/// implementations to be useful.
pub trait Encoder<D> {
    type Err: std::error::Error + 'static;

    /// Compute the values from the `input` and store the result in the initialized mutable `buffer`.
    ///
    /// Implementations are allowed to panic if `buffer.shape() != self.shape()`.
    fn encode<'f>(
        &self,
        input: &RecordBatch,
        buffer: ArrayViewMut<'f, f32, D>,
    ) -> Result<(), Self::Err>;

    /// The shape of the output of this encoder.
    fn shape(&self) -> D;
}

impl<'e, E, Dm> Encoder<Dm> for &'e E
where
    E: Encoder<Dm>,
{
    type Err = E::Err;

    fn encode<'f>(
        &self,
        input: &RecordBatch,
        buffer: ArrayViewMut<'f, f32, Dm>,
    ) -> Result<(), Self::Err> {
        <E as Encoder<Dm>>::encode(self, input, buffer)
    }

    fn shape(&self) -> Dm {
        <E as Encoder<Dm>>::shape(self)
    }
}

/// An [`Encoder`](Encoder) that simply counts how many rows in the input are present in a dictionary.
#[derive(Debug)]
pub struct Dictionary {
    dict: HashSet<String>,
}

impl Dictionary {
    /// Create a new dictionary from a collection of [`&str`](str).
    pub fn new<'a, I>(values: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        Self {
            dict: values.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn count<T: StringOffsetSizeTrait>(&self, data: &GenericStringArray<T>) -> u64 {
        data.into_iter()
            .filter_map(|m_s| m_s.and_then(|s| self.dict.get(s)))
            .count() as u64
    }
}

impl Encoder<Ix1> for Dictionary {
    type Err = Infallible;

    /// In the first column of `input`, count the number of rows which match an element of the
    /// dictionary.
    fn encode<'f>(
        &self,
        input: &RecordBatch,
        mut buffer: ArrayViewMut<'f, f32, Ix1>,
    ) -> Result<(), Self::Err> {
        let column = input.column(0);
        let count = match column.data_type() {
            DataType::Utf8 => {
                let sar: &GenericStringArray<i32> = column.as_any().downcast_ref().unwrap();
                Some(self.count(sar))
            }
            DataType::LargeUtf8 => {
                let sar: &GenericStringArray<i64> = column.as_any().downcast_ref().unwrap();
                Some(self.count(sar))
            }
            _ => None,
        }
        .and_then(|matches| {
            if column.len() != 0 {
                Some(matches as f32)
            } else {
                None
            }
        });
        *buffer.get_mut(0).unwrap() = count.unwrap_or(f32::NAN);
        Ok(())
    }

    fn shape(&self) -> Ix1 {
        Ix1(1)
    }
}

/// An [`Encoder`](Encoder) that horizontally stacks the output of a collection of [`Encoder`](Encoder)s.
pub struct StackedEncoder<D> {
    stack: Vec<D>,
    shape: Ix1,
}

impl<D> StackedEncoder<D>
where
    D: Encoder<Ix1>,
{
    /// Construct a new [`StackedEncoder`](StackedEncoder) from a collection of [`Encoder`](Encoder)s.
    pub fn from_vec(stack: Vec<D>) -> Self {
        let shape = stack.iter().map(|encoder| encoder.shape()[0]).sum();
        Self {
            stack,
            shape: Ix1(shape),
        }
    }
}

impl<D> Encoder<Ix1> for StackedEncoder<D>
where
    D: Encoder<Ix1>,
{
    type Err = D::Err;

    fn encode<'f>(
        &self,
        input: &RecordBatch,
        mut buffer: ArrayViewMut<'f, f32, Ix1>,
    ) -> Result<(), Self::Err> {
        let mut idx = 0usize;
        for encoder in self.stack.iter() {
            let next = idx + encoder.shape()[0];
            let sliced = buffer.slice_axis_mut(Axis(0), Slice::from(idx..next));
            encoder.encode(input, sliced)?;
            idx = next;
        }
        Ok(())
    }

    fn shape(&self) -> Ix1 {
        self.shape
    }
}

#[cfg(test)]
pub mod tests {
    use ndarray::{array, Array, Ix2};

    use std::iter::once;

    use super::{Dictionary, Encoder, StackedEncoder};
    use crate::tests::*;

    macro_rules! encode {
        ($encoder:ident, $input:ident, $output:expr) => {{
            $encoder.encode(&$input, $output).unwrap();
        }};
        ($encoder:ident, $input:ident) => {{
            let mut output = Array::zeros($encoder.shape());
            encode!($encoder, $input, output.view_mut());
            output
        }};
    }

    #[test]
    fn encoder_dictionary() {
        let last_names = <fake::locales::EN as fake::locales::Data>::NAME_LAST_NAME;
        let encoder = Dictionary::new(last_names.iter().copied());
        let names_array = string_array_of(fake::faker::name::en::LastName(), 1000);
        let input = record_batch_of(once(names_array));
        let output = encode!(encoder, input);
        assert_eq!(output, array![511.]);
    }

    #[test]
    fn encoder_stacked() {
        let data = [
            <fake::locales::EN as fake::locales::Data>::NAME_LAST_NAME,
            <fake::locales::EN as fake::locales::Data>::ADDRESS_COUNTRY_CODE,
            <fake::locales::EN as fake::locales::Data>::JOB_FIELD,
        ];
        let encoder = StackedEncoder::from_vec(
            data.iter()
                .map(|values| Dictionary::new(values.iter().copied()))
                .collect(),
        );

        let mut output = Array::zeros(Ix2(encoder.stack.len(), encoder.shape()[0]));
        let slices = output.outer_iter_mut();

        let inputs = vec![
            string_array_of(fake::faker::name::en::LastName(), 1000),
            string_array_of(fake::faker::address::en::CountryCode(), 1000),
            string_array_of(fake::faker::job::en::Field(), 1000),
        ];
        for (slice, array) in slices.zip(inputs) {
            let input = record_batch_of(vec![array]);
            encode!(encoder, input, slice);
        }

        assert_eq!(
            output,
            array![[511.0, 0.0, 0.0], [0.0, 511.0, 1.0], [0.0, 21.0, 500.0]]
        );
    }
}
