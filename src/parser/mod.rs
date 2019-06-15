mod ast;
mod parser_inner;

pub use ast::*;

use cpython::PyDict;
use cpython::PyString;
use cpython::Python;
use cpython::ToPyObject;

use crate::parser::parser_inner::{ParserInner, Rule};

pub struct SourceError {
    location: (usize, usize), // Line, column
    description: String,
}

impl ToString for SourceError {
    fn to_string(&self) -> String {
        let (line, col) = self.location;
        format!(
            "Error '{}' at Line: {}, Col: {}",
            self.description, line, col
        )
    }
}

pub enum Error {
    PestError(String),
    FailedPathLoad,
    SourceError(SourceError),
}

impl ToPyObject for Error {
    type ObjectType = PyString;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        PyString::new(py, "Error")
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::PestError(ref msg) => {
                let msg = format!("Parse error: {}", msg);
                println!("Error: {}", msg);
                msg
            },
            Error::FailedPathLoad => "Failed to load source.".to_string(),
            Error::SourceError(ref source_error) => source_error.to_string(),
        }
    }
}

pub fn parse<'a>(path: &str, name: &str, globals: &'a PyDict) -> Result<ast::Script<'a>, Error> {
    use pest::Parser;

    let mut script = ast::Script::new(name);
    let source = load_source(path)?;

    let pairs =
        ParserInner::parse(Rule::main, &source).map_err(|e| Error::PestError(e.to_string()))?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::statement => {
                let mut inner = pair.into_inner();
                let identifier: &str = inner
                    .next()
                    .expect("Failed to get statement identifier.")
                    .as_str();
                let mut statement_params: Vec<ast::StatementParameter> = vec![];
                while let Some(param) = inner.next() {
                    match param.as_rule() {
                        Rule::statement_parameter => {
                            let param_kind = param
                                .into_inner()
                                .next()
                                .expect("Failed to get param kind.");
                            let span = param_kind.as_span();
                            match param_kind.as_rule() {
                                Rule::statement_parameter_number => {
                                    let numstr = param_kind.as_str();
                                    let num = numstr.parse::<i64>().map_err(|_| {
                                        Error::SourceError(SourceError {
                                            location: span.start_pos().line_col(),
                                            description: "Invalid parameter number.".to_string(),
                                        })
                                    })?;
                                    statement_params.push(ast::StatementParameter::Number(num));
                                }
                                Rule::statement_parameter_string_register => {
                                    let string_reg = param_kind.as_str();
                                    let num_section = &string_reg["str.".len()..];
                                    let code = num_section.parse::<u8>().map_err(|_| {
                                        Error::SourceError(SourceError {
                                            location: span.start_pos().line_col(),
                                            description: "Invalid string register.".to_string(),
                                        })
                                    })?;
                                    statement_params
                                        .push(ast::StatementParameter::StringRegister(code));
                                }
                                Rule::statement_parameter_register => {
                                    let reg = param_kind.as_str();
                                    let num_section = &reg["reg.".len()..];
                                    let code = num_section.parse::<u8>().map_err(|_| {
                                        Error::SourceError(SourceError {
                                            location: span.start_pos().line_col(),
                                            description: "Invalid register.".to_string(),
                                        })
                                    })?;
                                    statement_params.push(ast::StatementParameter::Register(code, globals));
                                }
                                Rule::statement_parameter_position_register => {
                                    let pos_reg = param_kind.as_str();
                                    let num_section = &pos_reg["pos.".len()..];
                                    let code = num_section.parse::<u8>().map_err(|_| {
                                        Error::SourceError(SourceError {
                                            location: span.start_pos().line_col(),
                                            description: "Invalid position register.".to_string(),
                                        })
                                    })?;
                                    statement_params
                                        .push(ast::StatementParameter::PositionRegister(code));
                                }
                                Rule::statement_parameter_local_var => {
                                    let var = param_kind
                                        .into_inner()
                                        .next()
                                        .expect("Failed to get local variable.")
                                        .as_str();
                                    statement_params.push(ast::StatementParameter::LocalVariable(
                                        var.to_string(),
                                    ));
                                }
                                Rule::statement_parameter_global_var => {
                                    let var = param_kind
                                        .into_inner()
                                        .next()
                                        .expect("Failed to get global variable.")
                                        .as_str();
                                    statement_params.push(ast::StatementParameter::GlobalVariable(
                                        var.to_string(),
                                    ));
                                }
                                Rule::statement_parameter_autoprefixed_global_var => {
                                    let var = param_kind
                                        .into_inner()
                                        .next()
                                        .expect("Failed to get global variable.")
                                        .as_str();
                                    statement_params.push(
                                        ast::StatementParameter::AutoPrefixedGlobalVariable(
                                            var.to_string(),
                                        ),
                                    );
                                }
                                Rule::statement_parameter_id => {
                                    let param_id = param_kind
                                        .into_inner()
                                        .next()
                                        .expect("Failed to get id parameter.");

                                    let kind = match param_id.as_rule() {
                                        Rule::animation_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::AnimationId(text)
                                        }
                                        Rule::faction_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::FactionId(text)
                                        }
                                        Rule::info_page_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::InfoPageId(text)
                                        }
                                        Rule::item_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::ItemId(text)
                                        }
                                        Rule::map_icon_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::MapIconId(text)
                                        }
                                        Rule::game_menu_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::GameMenuId(text)
                                        }
                                        Rule::mesh_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::MeshId(text)
                                        }
                                        Rule::mission_template_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::MissionTemplateId(text)
                                        }
                                        Rule::particle_system_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::ParticleSystemId(text)
                                        }
                                        Rule::party_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::PartyId(text)
                                        }
                                        Rule::party_template_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::PartyTemplateId(text)
                                        }
                                        Rule::postfx_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::PostfxId(text)
                                        }
                                        Rule::presentation_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::PresentationId(text)
                                        }
                                        Rule::quest_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::QuestId(text)
                                        }
                                        Rule::scene_prop_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::ScenePropId(text)
                                        }
                                        Rule::scene_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::SceneId(text)
                                        }
                                        Rule::script_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::ScriptId(text)
                                        }
                                        Rule::skill_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::SkillId(text)
                                        }
                                        Rule::sound_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::SoundId(text)
                                        }
                                        Rule::string_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::StringId(text)
                                        }
                                        Rule::tableau_material_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::TableauMaterialId(text)
                                        }
                                        Rule::troop_id => {
                                            let text = param_id
                                                .into_inner()
                                                .next()
                                                .expect("Failed to get inner id parameter.")
                                                .as_str()
                                                .to_string();
                                            ast::StatementParameterId::TroopId(text)
                                        }
                                        _ => unreachable!(),
                                    };
                                    statement_params
                                        .push(ast::StatementParameter::StatementParamId(kind));
                                }
                                Rule::identifier => {
                                    let id = param_kind.as_str();
                                    statement_params.push(ast::StatementParameter::Identifier(
                                        id.to_string(),
                                        &globals,
                                    ));
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                let statement =
                    ast::Statement::new(identifier.to_string(), statement_params, &globals);
                script.push_statement(statement);
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    Ok(script)
}

fn load_source(path: &str) -> Result<String, Error> {
    use std::fs;
    fs::read_to_string(path).map_err(|_| Error::FailedPathLoad)
}
