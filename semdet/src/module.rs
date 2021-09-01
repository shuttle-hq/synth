use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;

use ndarray::{Array, Array2, ArrayView, Dimension, Ix1, Ix2};

use std::collections::HashMap;

use std::sync::Arc;

use crate::{Decoder, Encoder, Error};

/// A builder for [`Module`](Module).
#[derive(Default)]
pub struct ModuleBuilder<E = (), M = (), D = ()> {
    encoder: Option<E>,
    model: Option<M>,
    decoder: Option<D>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E, M, D> ModuleBuilder<E, M, D> {
    /// Set the [`Module`](Module)'s [`Encoder`](crate::Encoder).
    pub fn encoder<Dm, EE: Encoder<Dm>>(self, encoder: EE) -> ModuleBuilder<EE, M, D> {
        ModuleBuilder::<EE, M, D> {
            encoder: Some(encoder),
            model: self.model,
            decoder: self.decoder,
        }
    }

    /// Set the [`Module`](Module)'s [`Decoder`](crate::Decoder).
    pub fn decoder<Dm, DD: Decoder<Dm>>(self, decoder: DD) -> ModuleBuilder<E, M, DD> {
        ModuleBuilder::<E, M, DD> {
            encoder: self.encoder,
            model: self.model,
            decoder: Some(decoder),
        }
    }

    /// Set the [`Module`](Module)'s [`Model`](Model).
    pub fn model<Dm, MM: Model<Dm>>(self, model: MM) -> ModuleBuilder<E, MM, D> {
        ModuleBuilder::<E, MM, D> {
            encoder: self.encoder,
            model: Some(model),
            decoder: self.decoder,
        }
    }
}

impl<E, M, D> ModuleBuilder<E, M, D>
where
    E: Encoder<Ix1>,
    M: Model<Ix2, Output = Ix2>,
    D: Decoder<Ix1>,
{
    /// Build the [`Module`](Module).
    ///
    /// # Panics
    /// If one of encoder, model or decoder has not been set.
    fn build(self) -> Result<Module<E, M, D>, Error> {
        let encoder = self.encoder.expect("missing an encoder");

        let model = self.model.expect("missing a model");

        let decoder = self.decoder.expect("missing a decoder");

        Ok(Module {
            encoder,
            model,
            decoder,
        })
    }
}

/// Trait for functions that compute an output [`Array`](ndarray::Array) of variable shape from
/// an input [`ArrayView`](ndarray::ArrayView) of variable shape.
pub trait Model<D> {
    type Err: std::error::Error;
    type Output: Dimension;

    fn forward(&self, input: ArrayView<f32, D>) -> Result<Array<f32, Self::Output>, Self::Err>;
}

/// Put together an [`Encoder`](crate::Encoder), a [`Model`](Model) and a [`Decoder`](crate::Decoder)
/// into a pipeline that computes outputs from an input [`RecordBatch`](arrow::record_batch::RecordBatch).
pub struct Module<E = (), M = (), D = ()> {
    encoder: E,
    model: M,
    decoder: D,
}

impl Module {
    /// Create a new [`Module`](Module) builder.
    pub fn builder() -> ModuleBuilder {
        ModuleBuilder::new()
    }
}

impl<E, M, D> Module<E, M, D>
where
    E: Encoder<Ix1>,
    M: Model<Ix2, Output = Ix2, Err = Error>,
    D: Decoder<Ix1>,
{
    /// Compute the result of passing an input [`RecordBatch`](arrow::record_batch::RecordBatch)
    /// through the pipeline.
    ///
    /// Each column of `input` is passed separately to the [`Encoder`](crate::Encoder), [`Model`](Model)
    /// and [`Decoder`](crate::Decoder). The output [`Decoder::Value`](crate::Decoder::Value) are
    /// then assembled into a [`HashMap`](HashMap) with keys corresponding to `input` column names.
    pub fn forward(&self, input: &RecordBatch) -> Result<HashMap<String, D::Value>, Error> {
        let fields = input.schema().fields().clone();
        let columns = input.columns().iter().cloned();

        let mut buffer = Array2::zeros(Ix2(columns.len(), self.encoder.shape()[0]));
        let slices = buffer.outer_iter_mut();

        for ((field, column), slice) in fields.iter().cloned().zip(columns).zip(slices) {
            let column_input =
                RecordBatch::try_new(Arc::new(Schema::new(vec![field])), vec![column])?;
            self.encoder
                .encode(&column_input, slice)
                .map_err(Error::encoder)?;
        }

        let output = self.model.forward(buffer.view())?;

        let slices = output.outer_iter();
        let mut decoded = HashMap::new();
        for (field, row) in fields.into_iter().zip(slices) {
            decoded.insert(
                field.name().to_string(),
                self.decoder.decode(row).map_err(Error::decoder)?,
            );
        }
        Ok(decoded)
    }
}

pub mod dummy {
    //! A simple proof-of-concept pipeline
    use ndarray::{Array, ArrayView, Axis, Ix1, Ix2};

    use super::{Model, Module};
    use crate::decode::{Decoder, MaxIndexDecoder};
    use crate::encode::{Dictionary, Encoder, StackedEncoder};

    #[cfg(feature = "torch")]
    use ndarray::ArrayD;

    #[cfg(feature = "torch")]
    use tch::{jit::CModule, nn::Module as NnModule, TchError, Tensor};

    #[cfg(feature = "torch")]
    use std::{
        convert::{TryFrom, TryInto},
        io::{Cursor, Read},
    };

    #[cfg(feature = "torch")]
    static PRETRAINED: &[u8] = include_bytes!(env!("PRETRAINED"));

    use std::convert::Infallible;

    use crate::Error;

    macro_rules! dummy_feature {
        ($id:ident, $locale:ident) => {
            Dictionary::new(
                <fake::locales::$locale as fake::locales::Data>::$id
                    .into_iter()
                    .map(|s| *s),
            )
        };
    }

    macro_rules! dummies {
        {$(($id:ident, $locale:ident, $name:literal, $ty:path)$(,)?)+} => {
            pub fn encoder() -> impl Encoder<Ix1, Err = Infallible> + Send {
                StackedEncoder::from_vec(vec![
                    $(dummy_feature!($id, $locale),)+
                ])
            }

            pub fn decoder() -> impl Decoder<Ix1, Value = Option<&'static str>> + Send {
                MaxIndexDecoder::from_vec(vec![
                    $($name,)+
                ])
            }

            #[cfg(test)]
            pub mod tests {
                use arrow::record_batch::RecordBatch;

                use crate::tests::*;

                pub fn data(len: usize) -> RecordBatch {
                    record_batch_of_with_names(vec![
                        $(($name, string_array_of($ty(fake::locales::$locale), len)),)+
                    ])
                }
            }
        }
    }

    dummies! {
        (NAME_FIRST_NAME, EN, "name_first_name", fake::faker::name::raw::FirstName),
        (NAME_LAST_NAME, EN, "name_last_name", fake::faker::name::raw::LastName),
        (JOB_FIELD, EN, "job_field", fake::faker::job::raw::Field),
        (ADDRESS_COUNTRY, EN, "address_country", fake::faker::address::raw::CountryName),
        (ADDRESS_COUNTRY_CODE, EN, "address_country_code", fake::faker::address::raw::CountryCode),
        (ADDRESS_TIME_ZONE, EN, "address_time_zone", fake::faker::address::raw::TimeZone),
        (ADDRESS_STATE, EN, "address_state", fake::faker::address::raw::StateName),
        (ADDRESS_STATE_ABBR, EN, "address_state_abbr", fake::faker::address::raw::StateAbbr),
        (CURRENCY_NAME, EN, "currency_name", fake::faker::currency::raw::CurrencyName),
        (CURRENCY_CODE, EN, "currency_code", fake::faker::currency::raw::CurrencyCode)
    }

    #[cfg(feature = "torch")]
    pub struct DummyCModule(CModule);

    #[cfg(feature = "torch")]
    impl DummyCModule {
        pub fn load_data<R: Read>(buffer: &mut R) -> Result<Self, TchError> {
            let c_module = CModule::load_data(buffer)?;
            Ok(Self(c_module))
        }

        pub fn pretrained() -> Self {
            Self::load_data(&mut Cursor::new(PRETRAINED))
                .expect("failed to load pretrained module (torch backend)")
        }
    }

    #[cfg(feature = "torch")]
    impl Model<Ix2> for DummyCModule {
        type Err = Error;
        type Output = Ix2;

        fn forward(
            &self,
            input: ArrayView<f32, Ix2>,
        ) -> Result<Array<f32, Self::Output>, Self::Err> {
            let input_t = Tensor::try_from(input).map_err(Error::model)?;
            let output_t = self.0.forward(&input_t);
            let output: ArrayD<f32> = (&output_t).try_into()?;
            let output = output.into_dimensionality::<Ix2>()?;
            Ok(output)
        }
    }

    pub struct DummyNativeModule;

    impl Model<Ix2> for DummyNativeModule {
        type Err = Error;
        type Output = Ix2;

        fn forward(
            &self,
            input: ArrayView<f32, Ix2>,
        ) -> Result<Array<f32, Self::Output>, Self::Err> {
            let max = input.fold_axis(Axis(0), f32::NEG_INFINITY, |m, v| m.max(*v));
            let mut exp = ((-max) + input).mapv(f32::exp);
            let total = exp
                .map_axis(Axis(1), |view| view.sum())
                .into_shape(Ix2(exp.shape()[0], 1))
                .unwrap();
            exp = exp / total;
            Ok(exp)
        }
    }

    pub fn module() -> Module<
        impl Encoder<Ix1>,
        impl Model<Ix2, Output = Ix2, Err = Error>,
        impl Decoder<Ix1, Value = Option<&'static str>>,
    > {
        Module::builder()
            .encoder(encoder())
            .decoder(decoder())
            .model(DummyNativeModule)
            .build()
            .expect("failed to build dummy module (torch backend)")
    }

    #[cfg(feature = "torch")]
    pub fn module_torch() -> Module<
        impl Encoder<Ix1>,
        impl Model<Ix2, Output = Ix2, Err = Error>,
        impl Decoder<Ix1, Value = Option<&'static str>>,
    > {
        Module::builder()
            .encoder(encoder())
            .decoder(decoder())
            .model(DummyCModule::pretrained())
            .build()
            .expect("failed to build dummy module (native backend)")
    }
}

#[cfg(feature = "train")]
pub mod python_bindings {
    use arrow::{
        array::{Array as ArrowArrayTrait, StructArray},
        ffi::{ArrowArray, ArrowArrayRef},
        record_batch::RecordBatch,
    };

    use ndarray::{Array, Dimension};

    use pyo3::prelude::*;

    use std::convert::Infallible;

    use super::*;

    trait IntoPyRes {
        type Ok;
        fn or_else_raise(self) -> PyResult<Self::Ok>;
    }

    impl<T, E> IntoPyRes for Result<T, E>
    where
        E: std::error::Error,
    {
        type Ok = T;
        fn or_else_raise(self) -> PyResult<<Self as IntoPyRes>::Ok> {
            self.map_err(|arr_err| pyo3::panic::PanicException::new_err(arr_err.to_string()))
        }
    }

    pub unsafe fn import_record_batch(record_batch: &PyAny) -> PyResult<RecordBatch> {
        let (p_array, p_schema) = ArrowArray::into_raw(ArrowArray::empty());
        record_batch
            .getattr("_export_to_c")?
            .call1((p_array as usize, p_schema as usize))?;
        let c_array = ArrowArray::try_from_raw(p_array, p_schema).or_else_raise()?;
        let s_array = StructArray::from(c_array.to_data().or_else_raise()?);
        Ok(RecordBatch::from(&s_array))
    }

    pub unsafe fn export_record_batch(
        pyarrow: &PyModule,
        record_batch: RecordBatch,
    ) -> PyResult<&PyAny> {
        let struct_array = StructArray::from(record_batch);
        let (p_array, p_schema) = struct_array.to_raw().or_else_raise()?;
        let output = pyarrow
            .getattr("RecordBatch")?
            .getattr("_import_from_c")?
            .call1((p_array as usize, p_schema as usize))?;
        Ok(output)
    }

    #[pyclass(name = "Encoder")]
    struct BoundEncoder {
        encoder: Box<dyn Encoder<Ix1, Err = Infallible> + Send>,
    }

    #[pymethods]
    impl BoundEncoder {
        #[new]
        fn new() -> Self {
            Self {
                encoder: Box::new(dummy::encoder()),
            }
        }

        fn encode<'p>(&self, py: Python<'p>, record_batch: &PyAny) -> PyResult<&'p PyAny> {
            let record_batch = unsafe { import_record_batch(record_batch) }?;
            let shape = self.encoder.shape();
            let mut buffer = Array::zeros(shape);
            self.encoder
                .encode(&record_batch, buffer.view_mut())
                .or_else_raise()?;
            let as_py = buffer.into_raw_vec().into_py(py);
            let py_tensor = py
                .import("torch")?
                .getattr("FloatTensor")?
                .call1((as_py,))?;
            py_tensor.getattr("reshape")?.call1((shape.into_pattern(),))
        }
    }

    pub fn bind(py: Python) -> PyResult<&PyModule> {
        let module = PyModule::new(py, "dummy")?;
        module.add_class::<BoundEncoder>()?;
        Ok(module)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn dummy_end_to_end<E, M, D>(module: Module<E, M, D>)
    where
        E: Encoder<Ix1>,
        M: Model<Ix2, Output = Ix2, Err = Error>,
        D: Decoder<Ix1, Value = Option<&'static str>>,
    {
        let input = dummy::tests::data(1000);
        let output = module.forward(&input).unwrap();
        for (column, target) in output.into_iter() {
            assert_eq!(Some(column.as_str()), target)
        }
    }

    #[test]
    fn dummy_end_to_end_native() {
        let module = dummy::module();
        dummy_end_to_end(module)
    }

    #[cfg(feature = "torch")]
    #[test]
    fn dummy_end_to_end_torch() {
        let module = dummy::module_torch();
        dummy_end_to_end(module)
    }
}
