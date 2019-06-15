use cpython::{PyDict, PyInt, PyList, PyObject, PyString, PyTuple, Python, ToPyObject};

pub struct Script<'a> {
    name: String,
    statements: Vec<Statement<'a>>,
}

impl<'a> Script<'a> {
    pub(crate) fn new<T: ToString>(name: T) -> Self {
        Script {
            name: name.to_string(),
            statements: vec![],
        }
    }

    pub fn push_statement(&mut self, statement: Statement<'a>) {
        self.statements.push(statement);
    }
}

impl<'a> ToPyObject for Script<'a> {
    type ObjectType = PyTuple;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        use cpython::PythonObject;

        let statements: Vec<PyObject> = self
            .statements
            .iter()
            .map(|v| v.to_py_object(py).into_object())
            .collect();

        let statements = PyList::new(py, &statements);
        let name = PyString::new(py, &self.name);

        let script = vec![name.into_object(), statements.into_object()];
        PyTuple::new(py, &script)
    }
}

pub struct Statement<'a> {
    globals: &'a PyDict,
    operation: String,
    parameters: Vec<StatementParameter<'a>>,
}

impl<'a> Statement<'a> {
    pub fn new(
        operation: String,
        parameters: Vec<StatementParameter<'a>>,
        globals: &'a PyDict,
    ) -> Self {
        Self {
            globals,
            operation,
            parameters,
        }
    }
}

impl<'a> ToPyObject for Statement<'a> {
    type ObjectType = PyObject;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        use cpython::PythonObject;

        let op = self
            .globals
            .get_item(py, &self.operation)
            .unwrap_or_else(|| py.NotImplemented());
        let mut params: Vec<PyObject> =
            self.parameters.iter().map(|v| v.to_py_object(py)).collect();

        if params.is_empty() {
            op
        }
        else {
            let mut all = vec![op];
            all.append(&mut params);
            PyTuple::new(py, &all).into_object()
        }
    }
}

pub enum StatementParameter<'a> {
    Identifier(String, &'a PyDict),
    Register(u8, &'a PyDict),
    StringRegister(u8),
    PositionRegister(u8),
    LocalVariable(String),
    GlobalVariable(String),
    AutoPrefixedGlobalVariable(String),
    StatementParamId(StatementParameterId),
    Number(i64),
}

fn get_register(py: Python, code: u8, globals: &PyDict) -> PyObject {
    let key = format!("reg{}", code);
    globals.get_item(py, &key).unwrap_or_else(|| py.NotImplemented())
}

impl<'a> ToPyObject for StatementParameter<'a> {
    type ObjectType = PyObject;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        use cpython::PythonObject;
        use std::os::raw::c_long;
        match self {
            StatementParameter::Identifier(ref id, globals) => globals
                .get_item(py, id)
                .unwrap_or_else(|| py.NotImplemented()),
            StatementParameter::Register(reg_code, dict) => get_register(py, *reg_code, dict),
            StatementParameter::StringRegister(reg_code) => {
                PyInt::new(py, *reg_code as c_long).into_object()
            }
            StatementParameter::PositionRegister(reg_code) => {
                PyInt::new(py, *reg_code as c_long).into_object()
            }
            StatementParameter::LocalVariable(ref id) => {
                let value = format!(":{}", id);
                PyString::new(py, &value).into_object()
            }
            StatementParameter::GlobalVariable(ref id) => {
                let value = format!("${}", id);
                PyString::new(py, &value).into_object()
            }
            StatementParameter::AutoPrefixedGlobalVariable(ref id) => {
                let value = format!("$g_{}", id);
                PyString::new(py, &value).into_object()
            }
            StatementParameter::StatementParamId(ref param_id) => {
                param_id.to_py_object(py).into_object()
            }
            StatementParameter::Number(num) => PyInt::new(py, *num as c_long).into_object(),
        }
    }
}

pub enum StatementParameterId {
    AnimationId(String),
    FactionId(String),
    InfoPageId(String),
    ItemId(String),
    MapIconId(String),
    GameMenuId(String),
    MeshId(String),
    MissionTemplateId(String),
    ParticleSystemId(String),
    PartyId(String),
    PartyTemplateId(String),
    PostfxId(String),
    PresentationId(String),
    QuestId(String),
    ScenePropId(String),
    SceneId(String),
    ScriptId(String),
    SkillId(String),
    SoundId(String),
    StringId(String),
    TableauMaterialId(String),
    TroopId(String),
}

impl ToPyObject for StatementParameterId {
    type ObjectType = PyString;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        PyString::new(
            py,
            &match self {
                StatementParameterId::AnimationId(ref id) => format!("anim_{}", id),
                StatementParameterId::FactionId(ref id) => format!("fac_{}", id),
                StatementParameterId::InfoPageId(ref id) => format!("id_{}", id),
                StatementParameterId::ItemId(ref id) => format!("itm_{}", id),
                StatementParameterId::MapIconId(ref id) => format!("icon_{}", id),
                StatementParameterId::GameMenuId(ref id) => format!("menu_{}", id),
                StatementParameterId::MeshId(ref id) => format!("mesh_{}", id),
                StatementParameterId::MissionTemplateId(ref id) => format!("mst_{}", id),
                StatementParameterId::ParticleSystemId(ref id) => format!("psys_{}", id),
                StatementParameterId::PartyId(ref id) => format!("p_{}", id),
                StatementParameterId::PartyTemplateId(ref id) => format!("pt_{}", id),
                StatementParameterId::PostfxId(ref id) => format!("pfx_{}", id),
                StatementParameterId::PresentationId(ref id) => format!("prsnt_{}", id),
                StatementParameterId::QuestId(ref id) => format!("qst_{}", id),
                StatementParameterId::ScenePropId(ref id) => format!("spr_{}", id),
                StatementParameterId::SceneId(ref id) => format!("scn_{}", id),
                StatementParameterId::ScriptId(ref id) => format!("script_{}", id),
                StatementParameterId::SkillId(ref id) => format!("skl_{}", id),
                StatementParameterId::SoundId(ref id) => format!("snd_{}", id),
                StatementParameterId::StringId(ref id) => format!("str_{}", id),
                StatementParameterId::TableauMaterialId(ref id) => format!("tableau_{}", id),
                StatementParameterId::TroopId(ref id) => format!("trp_{}", id),
            },
        )
    }
}
