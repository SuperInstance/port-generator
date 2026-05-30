//! Cross-language port generator from Rust API definitions.
//!
//! Reads a Rust library's public API and generates equivalent code in
//! C, Python, TypeScript, Go, and Zig. Used to keep the Grand Pattern
//! polyglot toolkit in sync across 16+ languages.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A type in the target language.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortType {
    F64,
    F32,
    I64,
    I32,
    Usize,
    Bool,
    StringType,
    Vec(Box<PortType>),
    Option(Box<PortType>),
    Tuple(Vec<PortType>),
    Custom(String),
}

/// A function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub typ: PortType,
}

/// A function signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortFn {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<PortType>,
    pub doc: Option<String>,
}

/// A struct definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortStruct {
    pub name: String,
    pub fields: Vec<Param>,
    pub doc: Option<String>,
}

/// A trait definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTrait {
    pub name: String,
    pub methods: Vec<PortFn>,
    pub doc: Option<String>,
}

/// A complete API surface to port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortApi {
    pub name: String,
    pub version: String,
    pub structs: Vec<PortStruct>,
    pub traits: Vec<PortTrait>,
    pub functions: Vec<PortFn>,
}

/// Target language for code generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetLang {
    C,
    Python,
    TypeScript,
    Go,
    Zig,
}

impl fmt::Display for TargetLang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetLang::C => write!(f, "c"),
            TargetLang::Python => write!(f, "python"),
            TargetLang::TypeScript => write!(f, "typescript"),
            TargetLang::Go => write!(f, "go"),
            TargetLang::Zig => write!(f, "zig"),
        }
    }
}

/// Port type to C type string.
pub fn c_type(t: &PortType) -> String {
    match t {
        PortType::F64 => "double".into(),
        PortType::F32 => "float".into(),
        PortType::I64 => "int64_t".into(),
        PortType::I32 => "int32_t".into(),
        PortType::Usize => "size_t".into(),
        PortType::Bool => "bool".into(),
        PortType::StringType => "const char*".into(),
        PortType::Vec(inner) => format!("{}*", c_type(inner)),
        PortType::Option(inner) => c_type(inner),
        PortType::Tuple(_) => "void*".into(),
        PortType::Custom(name) => name.clone(),
    }
}

/// Port type to Python type string.
pub fn py_type(t: &PortType) -> String {
    match t {
        PortType::F64 | PortType::F32 => "float".into(),
        PortType::I64 | PortType::I32 | PortType::Usize => "int".into(),
        PortType::Bool => "bool".into(),
        PortType::StringType => "str".into(),
        PortType::Vec(inner) => format!("list[{}]", py_type(inner)),
        PortType::Option(inner) => format!("{} | None", py_type(inner)),
        PortType::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(py_type).collect();
            format!("tuple[{}]", parts.join(", "))
        }
        PortType::Custom(name) => format!("\"{name}\""),
    }
}

/// Port type to TypeScript type string.
pub fn ts_type(t: &PortType) -> String {
    match t {
        PortType::F64 | PortType::F32 => "number".into(),
        PortType::I64 | PortType::I32 | PortType::Usize => "number".into(),
        PortType::Bool => "boolean".into(),
        PortType::StringType => "string".into(),
        PortType::Vec(inner) => format!("{}[]", ts_type(inner)),
        PortType::Option(inner) => format!("{} | null", ts_type(inner)),
        PortType::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(ts_type).collect();
            format!("[{}]", parts.join(", "))
        }
        PortType::Custom(name) => name.clone(),
    }
}

/// Port type to Go type string.
pub fn go_type(t: &PortType) -> String {
    match t {
        PortType::F64 => "float64".into(),
        PortType::F32 => "float32".into(),
        PortType::I64 => "int64".into(),
        PortType::I32 => "int32".into(),
        PortType::Usize => "uint".into(),
        PortType::Bool => "bool".into(),
        PortType::StringType => "string".into(),
        PortType::Vec(inner) => format!("[]{}", go_type(inner)),
        PortType::Option(inner) => format!("*{}", go_type(inner)),
        PortType::Tuple(_) => "interface{}".into(),
        PortType::Custom(name) => name.clone(),
    }
}

/// Port type to Zig type string.
pub fn zig_type(t: &PortType) -> String {
    match t {
        PortType::F64 => "f64".into(),
        PortType::F32 => "f32".into(),
        PortType::I64 => "i64".into(),
        PortType::I32 => "i32".into(),
        PortType::Usize => "usize".into(),
        PortType::Bool => "bool".into(),
        PortType::StringType => "[]const u8".into(),
        PortType::Vec(inner) => format!("[]{}", zig_type(inner)),
        PortType::Option(inner) => format!("?{}", zig_type(inner)),
        PortType::Tuple(_) => "anytype".into(),
        PortType::Custom(name) => name.clone(),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl PortApi {
    /// Load from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Generate C header.
    pub fn generate_c(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "// Generated from {} v{}\n",
            self.name, self.version
        ));
        out.push_str("#include <stdint.h>\n#include <stdbool.h>\n\n");
        out.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");
        for s in &self.structs {
            if let Some(doc) = &s.doc {
                for line in doc.lines() {
                    out.push_str(&format!("// {line}\n"));
                }
            }
            out.push_str("typedef struct {\n");
            for f in &s.fields {
                out.push_str(&format!("    {} {};\n", c_type(&f.typ), f.name));
            }
            out.push_str(&format!("}} {};\n\n", s.name));
        }
        for f in &self.functions {
            if let Some(doc) = &f.doc {
                for line in doc.lines() {
                    out.push_str(&format!("// {line}\n"));
                }
            }
            let ret = f
                .return_type
                .as_ref()
                .map(c_type)
                .unwrap_or_else(|| "void".into());
            let params: Vec<String> = f
                .params
                .iter()
                .map(|p| format!("{} {}", c_type(&p.typ), p.name))
                .collect();
            out.push_str(&format!(
                "{} {}({});\n\n",
                ret,
                f.name,
                params.join(", ")
            ));
        }
        out.push_str("#ifdef __cplusplus\n}\n#endif\n");
        out
    }

    /// Generate Python module.
    pub fn generate_python(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\"\"\"Generated from {} v{}\"\"\"\nfrom __future__ import annotations\nfrom dataclasses import dataclass\nfrom typing import Optional\n\n",
            self.name, self.version
        ));
        for s in &self.structs {
            if let Some(doc) = &s.doc {
                out.push_str(&format!("{}\n", doc));
            }
            out.push_str(&format!("@dataclass\nclass {}:\n", s.name));
            for f in &s.fields {
                out.push_str(&format!("    {}: {}\n", f.name, py_type(&f.typ)));
            }
            out.push('\n');
        }
        for f in &self.functions {
            if let Some(doc) = &f.doc {
                out.push_str(&format!("    \"\"\"{}\"\"\"\n", doc.trim()));
            }
            let params: Vec<String> = f
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, py_type(&p.typ)))
                .collect();
            let ret = f
                .return_type
                .as_ref()
                .map(|t| format!(" -> {}", py_type(t)))
                .unwrap_or_default();
            out.push_str(&format!(
                "def {}({}){}:\n    ...\n\n",
                f.name,
                params.join(", "),
                ret
            ));
        }
        out
    }

    /// Generate TypeScript module.
    pub fn generate_typescript(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "// Generated from {} v{}\n\n",
            self.name, self.version
        ));
        for s in &self.structs {
            if let Some(doc) = &s.doc {
                out.push_str(&format!("/** {} */\n", doc));
            }
            out.push_str(&format!("export interface {} {{\n", s.name));
            for f in &s.fields {
                out.push_str(&format!("  {}: {};\n", f.name, ts_type(&f.typ)));
            }
            out.push_str("}\n\n");
        }
        for f in &self.functions {
            if let Some(doc) = &f.doc {
                out.push_str(&format!("/** {} */\n", doc));
            }
            let params: Vec<String> = f
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, ts_type(&p.typ)))
                .collect();
            let ret = f
                .return_type
                .as_ref()
                .map(|t| format!(": {}", ts_type(t)))
                .unwrap_or_default();
            out.push_str(&format!(
                "export function {}({}){};\n\n",
                f.name,
                params.join(", "),
                ret
            ));
        }
        out
    }

    /// Generate Go package.
    pub fn generate_go(&self) -> String {
        let mut out = String::new();
        let pkg = self.name.replace("-", "_");
        out.push_str(&format!(
            "// Generated from {} v{}\npackage {}\n\n",
            self.name, self.version, pkg
        ));
        for s in &self.structs {
            if let Some(doc) = &s.doc {
                out.push_str(&format!("// {}\n", doc));
            }
            out.push_str(&format!("type {} struct {{\n", s.name));
            for f in &s.fields {
                out.push_str(&format!(
                    "\t{} {}\n",
                    capitalize(&f.name),
                    go_type(&f.typ)
                ));
            }
            out.push_str("}\n\n");
        }
        for f in &self.functions {
            if let Some(doc) = &f.doc {
                out.push_str(&format!("// {}\n", doc));
            }
            let params: Vec<String> = f
                .params
                .iter()
                .map(|p| format!("{} {}", p.name, go_type(&p.typ)))
                .collect();
            let ret = f
                .return_type
                .as_ref()
                .map(go_type)
                .unwrap_or_default();
            if ret.is_empty() {
                out.push_str(&format!("func {}({}) {{}}\n\n", f.name, params.join(", ")));
            } else {
                out.push_str(&format!(
                    "func {}({}) {} {{}}\n\n",
                    f.name,
                    params.join(", "),
                    ret
                ));
            }
        }
        out
    }

    /// Generate Zig module.
    pub fn generate_zig(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "// Generated from {} v{}\n\n",
            self.name, self.version
        ));
        for s in &self.structs {
            if let Some(doc) = &s.doc {
                out.push_str(&format!("/// {}\n", doc));
            }
            out.push_str(&format!("pub const {} = struct {{\n", s.name));
            for f in &s.fields {
                out.push_str(&format!("    {}: {},\n", f.name, zig_type(&f.typ)));
            }
            out.push_str("};\n\n");
        }
        for f in &self.functions {
            if let Some(doc) = &f.doc {
                out.push_str(&format!("/// {}\n", doc));
            }
            let params: Vec<String> = f
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, zig_type(&p.typ)))
                .collect();
            let ret = f
                .return_type
                .as_ref()
                .map(|t| format!(" -> {}", zig_type(t)))
                .unwrap_or_default();
            out.push_str(&format!(
                "pub fn {}({}) {} {{}}\n\n",
                f.name,
                params.join(", "),
                ret
            ));
        }
        out
    }

    /// Generate for all languages.
    pub fn generate_all(&self) -> Vec<(TargetLang, String)> {
        vec![
            (TargetLang::C, self.generate_c()),
            (TargetLang::Python, self.generate_python()),
            (TargetLang::TypeScript, self.generate_typescript()),
            (TargetLang::Go, self.generate_go()),
            (TargetLang::Zig, self.generate_zig()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_api() -> PortApi {
        PortApi {
            name: "test-lib".into(),
            version: "0.1.0".into(),
            structs: vec![PortStruct {
                name: "Config".into(),
                fields: vec![
                    Param {
                        name: "threshold".into(),
                        typ: PortType::F64,
                    },
                    Param {
                        name: "name".into(),
                        typ: PortType::StringType,
                    },
                    Param {
                        name: "items".into(),
                        typ: PortType::Vec(Box::new(PortType::I32)),
                    },
                ],
                doc: Some("Configuration".into()),
            }],
            traits: vec![],
            functions: vec![
                PortFn {
                    name: "init".into(),
                    params: vec![Param {
                        name: "config".into(),
                        typ: PortType::Custom("Config".into()),
                    }],
                    return_type: Some(PortType::Bool),
                    doc: Some("Initialize".into()),
                },
                PortFn {
                    name: "process".into(),
                    params: vec![
                        Param {
                            name: "data".into(),
                            typ: PortType::Vec(Box::new(PortType::F64)),
                        },
                        Param {
                            name: "count".into(),
                            typ: PortType::Usize,
                        },
                    ],
                    return_type: Some(PortType::Vec(Box::new(PortType::F64))),
                    doc: None,
                },
            ],
        }
    }

    #[test]
    fn test_c_generation() {
        let api = sample_api();
        let c = api.generate_c();
        assert!(c.contains("typedef struct"));
        assert!(c.contains("double threshold"));
        assert!(c.contains("bool init"));
        assert!(c.contains("stdint.h"));
    }

    #[test]
    fn test_python_generation() {
        let api = sample_api();
        let py = api.generate_python();
        assert!(py.contains("@dataclass"));
        assert!(py.contains("class Config"));
        assert!(py.contains("threshold: float"));
        assert!(py.contains("items: list[int]"));
    }

    #[test]
    fn test_typescript_generation() {
        let api = sample_api();
        let ts = api.generate_typescript();
        assert!(ts.contains("export interface Config"));
        assert!(ts.contains("threshold: number"));
        assert!(ts.contains("items: number[]"));
    }

    #[test]
    fn test_go_generation() {
        let api = sample_api();
        let go = api.generate_go();
        assert!(go.contains("type Config struct"));
        assert!(go.contains("Threshold float64"));
        assert!(go.contains("package test_lib"));
    }

    #[test]
    fn test_zig_generation() {
        let api = sample_api();
        let zig = api.generate_zig();
        assert!(zig.contains("pub const Config = struct"));
        assert!(zig.contains("threshold: f64"));
    }

    #[test]
    fn test_json_roundtrip() {
        let api = sample_api();
        let json = api.to_json();
        let restored = PortApi::from_json(&json).unwrap();
        assert_eq!(restored.name, "test-lib");
        assert_eq!(restored.structs.len(), 1);
        assert_eq!(restored.functions.len(), 2);
    }

    #[test]
    fn test_generate_all() {
        let api = sample_api();
        let all = api.generate_all();
        assert_eq!(all.len(), 5);
        for (lang, code) in &all {
            assert!(!code.is_empty(), "Empty output for {lang}");
        }
    }

    #[test]
    fn test_c_option_type() {
        assert_eq!(c_type(&PortType::Option(Box::new(PortType::F64))), "double");
    }

    #[test]
    fn test_python_option_type() {
        assert_eq!(
            py_type(&PortType::Option(Box::new(PortType::F64))),
            "float | None"
        );
    }

    #[test]
    fn test_ts_vec_type() {
        assert_eq!(
            ts_type(&PortType::Vec(Box::new(PortType::Bool))),
            "boolean[]"
        );
    }

    #[test]
    fn test_go_option_type() {
        assert_eq!(
            go_type(&PortType::Option(Box::new(PortType::F64))),
            "*float64"
        );
    }

    #[test]
    fn test_zig_string_type() {
        assert_eq!(zig_type(&PortType::StringType), "[]const u8");
    }

    #[test]
    fn test_empty_api() {
        let api = PortApi {
            name: "empty".into(),
            version: "0.1.0".into(),
            structs: vec![],
            traits: vec![],
            functions: vec![],
        };
        let c = api.generate_c();
        assert!(c.contains("Generated from empty"));
        let py = api.generate_python();
        assert!(py.contains("Generated from empty"));
    }

    #[test]
    fn test_tuple_type_mapping() {
        let t = PortType::Tuple(vec![PortType::F64, PortType::I32]);
        assert_eq!(ts_type(&t), "[number, number]");
        assert_eq!(py_type(&t), "tuple[float, int]");
        assert_eq!(go_type(&t), "interface{}");
    }

    #[test]
    fn test_custom_type() {
        let t = PortType::Custom("MyStruct".into());
        assert_eq!(c_type(&t), "MyStruct");
        assert_eq!(py_type(&t), "\"MyStruct\"");
        assert_eq!(ts_type(&t), "MyStruct");
    }

    #[test]
    fn test_target_lang_display() {
        assert_eq!(TargetLang::C.to_string(), "c");
        assert_eq!(TargetLang::Python.to_string(), "python");
        assert_eq!(TargetLang::Zig.to_string(), "zig");
    }
}
