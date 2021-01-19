use anyhow::Result;

use pyo3::{
    types::IntoPyDict, FromPyObject, GILGuard, PyAny, PyObject, PyResult, Python, ToPyObject,
};

use std::collections::HashMap;

pub struct FakerGenerateBuilder<'a> {
    faker: &'a Faker,
    gil: GILGuard,
    generator: String,
    kwargs: HashMap<String, PyObject>,
}

impl<'a> FakerGenerateBuilder<'a> {
    fn new<R: AsRef<str>>(faker: &'a Faker, gil: GILGuard, generator: R) -> Self {
        // @brokad: validate generator!
        FakerGenerateBuilder {
            faker,
            gil,
            generator: generator.as_ref().to_string(),
            kwargs: HashMap::new(),
        }
    }

    pub fn arg<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: ToPyObject,
    {
        let key_str = key.as_ref().to_string();
        let value_obj = value.to_object(self.gil.python());
        self.kwargs.insert(key_str, value_obj);
        self
    }

    pub fn generate<R>(self) -> PyResult<R>
    where
        for<'r> R: FromPyObject<'r>,
    {
        self.faker
            .generate(self.gil.python(), &self.generator, &self.kwargs)
    }
}

#[derive(Clone)]
struct Faker(String);

impl Faker {
    fn new(py: Python<'_>) -> PyResult<Self> {
        let faker = py.import("faker")?.get("Faker")?.call0()?;
        let name = "faker".to_string();
        py.import("__main__")?.setattr(&name, faker)?;
        Ok(Self(name))
    }

    fn prepare<'a, R: AsRef<str>>(
        &'a self,
        py: GILGuard,
        generator: R,
    ) -> FakerGenerateBuilder<'a> {
        FakerGenerateBuilder::new(self, py, generator)
    }

    fn inner_py<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        py.import("__main__").and_then(|main| main.getattr(&self.0))
    }

    fn generate<'p, KW, R>(&self, py: Python<'p>, generator: &str, kwargs: KW) -> PyResult<R>
    where
        KW: IntoPyDict,
        R: FromPyObject<'p>,
    {
        self.inner_py(py)?
            .call_method(generator, (), Some(kwargs.into_py_dict(py)))?
            .extract()
    }
}

#[derive(Clone)]
pub struct Pythonizer {
    faker: Faker,
}

impl Pythonizer {
    pub fn new() -> Result<Self> {
        let gil = Python::acquire_gil();
        let faker = Faker::new(gil.python())?;
        Ok(Self { faker })
    }

    pub fn faker<R: AsRef<str>>(&self, generator: R) -> FakerGenerateBuilder<'_> {
        let gil = Python::acquire_gil();
        self.faker.prepare(gil, generator)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[async_std::test]
    async fn test_pythonizer() -> Result<()> {
        let pythonizer = Pythonizer::new()?;
        let res: String = pythonizer
            .faker("text")
            .arg("max_nb_chars", 10)
            .generate()?;
        assert!(res.len() <= 10);
        Ok(())
    }
}
