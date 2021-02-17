use super::super::prelude::*;

use pyo3::{
    types::IntoPyDict, types::PyTuple, FromPyObject, PyAny, PyObject, PyResult, Python, ToPyObject,
};

pub struct RandFaker {
    pub generator: String,
    pub faker_instance_name: String,
    pub args: HashMap<String, PyObject>,
}

impl RandFaker {
    pub(crate) fn new(
        generator: String,
        args: HashMap<String, FakerContentArgument>,
        locales: Vec<String>,
    ) -> Result<Self, anyhow::Error> {
        let gil = Python::acquire_gil();
        let python = gil.python();
        let faker = Self::faker(&locales, &python)?;
        let faker_instance_name = Self::instance_name();
        let main = python.import("__main__")?;
        main.setattr(&faker_instance_name, faker)?;
        Ok(RandFaker {
            generator,
            args: Self::map_args(args),
            faker_instance_name,
        })
    }

    /// Creates an instance of python `Faker` with appropriate locales
    fn faker<'p>(locales: &Vec<String>, python: &Python<'p>) -> PyResult<&'p PyAny> {
        let faker = python.import("faker")?.get("Faker")?;
        match locales.len() {
            // No locales call an empty constructor
            0 => faker.call0(),
            // Pass locales in python `Faker` constructor
            _ => faker.call1(PyTuple::new(python.clone(), vec![locales])),
        }
    }

    /// Maps FakerContentArgs to args which can be used by Python interpreter
    fn map_args<K, V>(args: HashMap<K, V>) -> HashMap<String, PyObject>
    where
        K: AsRef<str>,
        V: ToPyObject,
    {
        let gil = Python::acquire_gil();
        let python = gil.python();
        let mut kwargs = HashMap::new();
        for (key, value) in args {
            let key_str = key.as_ref().to_string();
            let value_obj = value.to_object(python);
            kwargs.insert(key_str, value_obj);
        }
        kwargs
    }

    /// Instance name is just an uuid which is then used for GC in the drop Impl
    /// Instance name starts with `_` and has no `-` to be a valid variable name in python
    fn instance_name() -> String {
        format!("_{}", uuid::Uuid::new_v4().to_simple().to_string())
    }

    fn generate<'p, R>(&self, python: Python<'p>) -> PyResult<R>
    where
        R: FromPyObject<'p>,
    {
        let faker_instance = python
            .import("__main__")
            .and_then(|main| main.getattr(&self.faker_instance_name))?;
        faker_instance
            .call_method(
                &self.generator,
                (),
                Some(self.args.clone().into_py_dict(python)),
            )?
            .extract()
    }
}

impl Drop for RandFaker {
    /// This essentially implements a garbage collector, cleaning up resources when
    /// RandFaker is dropped and the instance of python `Faker` is no longer used.
    fn drop(&mut self) {
        let gil = Python::acquire_gil();
        let python = gil.python();
        let code = format!("del {}", self.faker_instance_name);
        let _ = python.run(&code, None, None).map_err(|e| {
            error!(
                "Could not garbage collect variable {} in python runtime with error: {}",
                self.faker_instance_name, e
            );
            e
        });
    }
}

impl Generator for RandFaker {
    type Yield = String;

    type Return = Result<Never, Error>;

    fn next<R: Rng>(&mut self, _rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let gil = Python::acquire_gil();
        match self.generate::<String>(gil.python()) {
	    Ok(output) => GeneratorState::Yielded(output),
	    Err(err) => GeneratorState::Complete(Err(failed_crate!(
		target: Release,
		"a call to faker failed: {}",
		err
	    )))
	}
    }
}
