extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate cpython;

mod parser;

use cpython::{PyDict, PyErr, PyModule, PyResult, PyTuple, Python};

py_module_initializer!(
    mb_ext_script,
    initmb_ext_script,
    PyInit_mb_ext_script,
    |py, m| {
        m.add(
            py,
            "__doc__",
            "This module parses and maps .mbs script files as an extension to the M&B modsys.",
        )?;
        m.add(py, "version", py_fn!(py, version_py()))?;
        m.add(
            py,
            "parse",
            py_fn!(py, parse_py(path: &str, name: &str, modules: Vec<String>)),
        )?;
        Ok(())
    }
);

fn version_py(py: Python) -> PyResult<String> {
    let sys = py.import("sys")?;
    let version: String = sys.get(py, "version")?.extract(py)?;
    Ok(format!("{}-{}", "0.1.0", version))
}

fn parse_py(py: Python, path: &str, name: &str, modules: Vec<String>) -> PyResult<PyTuple> {
    use cpython::exc::Exception;
    use cpython::ToPyObject;

    let mut imports = vec![];

    for module_name in modules.iter() {
        let module = py.import(module_name)?;
        imports.push(module);
    }

    let globals = build_globals_dict(py, imports)?;

    match parser::parse(path, name, &globals) {
        Ok(script) => Ok(script.to_py_object(py)),
        Err(e) => Err(PyErr::new::<Exception, _>(py, e.to_string())),
    }
}

fn build_globals_dict(py: Python, imports: Vec<PyModule>) -> PyResult<PyDict> {
    let globals = PyDict::new(py);

    for module in imports.iter() {
        let dict = module.get(py, "__dict__")?.cast_into::<PyDict>(py)?;
        for (key, val) in dict.items(py).iter() {
            globals.set_item(py, key, val)?;
        }
    }

    Ok(globals)
}
