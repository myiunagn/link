//! linkc_bindgen —— 多语言绑定生成器
//!
//! 从 Link 源文件的 `export "<lang>" { ... }` 块生成目标语言的绑定代码:
//!
//! - `export "C"`          → C 头文件 (`.h`)
//! - `export "python"`     → Python 类型存根 (`.pyi`)
//! - `export "typescript"` → TypeScript 声明 (`.d.ts`)
//!
//! 用法:
//! ```ignore
//! use linkc_bindgen::{generate, TargetLang};
//!
//! let code = generate(&program, TargetLang::C, "my_module")?;
//! println!("{}", code);
//! ```

use linkc_parser::{FnSignature, Program, Stmt, TypeAnnotation};

mod c;
mod python;
mod typescript;

pub use c::CGenerator;
pub use python::PythonGenerator;
pub use typescript::TypeScriptGenerator;

/// 目标语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLang {
    C,
    Python,
    TypeScript,
}

impl TargetLang {
    /// 从字符串解析目标语言(大小写不敏感)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "c" | "header" | "h" => Some(TargetLang::C),
            "python" | "py" | "pyi" => Some(TargetLang::Python),
            "typescript" | "ts" | "dts" => Some(TargetLang::TypeScript),
            _ => None,
        }
    }

    /// 该语言对应的文件扩展名(不带前导点)
    pub fn extension(&self) -> &'static str {
        match self {
            TargetLang::C => "h",
            TargetLang::Python => "pyi",
            TargetLang::TypeScript => "d.ts",
        }
    }

    /// 与 `export "<lang>"` 块中 language 字符串匹配(大小写不敏感)
    pub fn matches_export_lang(&self, lang: &str) -> bool {
        let normalized = lang.to_lowercase();
        match self {
            TargetLang::C => normalized == "c",
            TargetLang::Python => normalized == "python" || normalized == "py",
            TargetLang::TypeScript => normalized == "typescript" || normalized == "ts",
        }
    }
}

/// 绑定生成器 trait:每种语言实现该接口
pub trait Generator {
    /// 生成绑定代码
    ///
    /// - `module_name`: 模块名(用于头文件宏前缀、Python 模块名等)
    /// - `module_path`: `export "<lang>" module "<path>"` 中的路径(可选)
    /// - `decls`: 所有导出的函数签名
    fn generate(
        &self,
        module_name: &str,
        module_path: Option<&str>,
        decls: &[FnSignature],
    ) -> String;
}

/// 从 Program 中收集所有匹配目标语言的 export 声明
///
/// 返回元组列表: `(module_path, decls)`
pub fn collect_exports(program: &Program, lang: TargetLang) -> Vec<(Option<String>, Vec<FnSignature>)> {
    let stmts = match program {
        Program::Block(s) => s,
    };
    let mut out = Vec::new();
    for stmt in stmts {
        if let Stmt::ExportDecl { language, module, decls } = stmt {
            if lang.matches_export_lang(language) {
                out.push((module.clone(), decls.clone()));
            }
        }
    }
    out
}

/// 便捷入口:为指定目标语言生成绑定代码
///
/// 如果源文件中有多个匹配的 export 块,会合并所有声明一起生成。
pub fn generate(program: &Program, lang: TargetLang, module_name: &str) -> Result<String, String> {
    let groups = collect_exports(program, lang);
    if groups.is_empty() {
        return Err(format!(
            "No `export \"{}\" {{ ... }}` block found in source",
            match lang {
                TargetLang::C => "C",
                TargetLang::Python => "python",
                TargetLang::TypeScript => "typescript",
            }
        ));
    }

    // 合并所有 export 块的声明(module_path 取第一个非空的)
    let mut all_decls = Vec::new();
    let mut merged_module_path: Option<String> = None;
    for (module_path, decls) in groups {
        if merged_module_path.is_none() && module_path.is_some() {
            merged_module_path = module_path;
        }
        all_decls.extend(decls);
    }

    let generator: Box<dyn Generator> = match lang {
        TargetLang::C => Box::new(CGenerator),
        TargetLang::Python => Box::new(PythonGenerator),
        TargetLang::TypeScript => Box::new(TypeScriptGenerator),
    };

    Ok(generator.generate(module_name, merged_module_path.as_deref(), &all_decls))
}

/// 类型映射辅助:将 Link `TypeAnnotation` 转换为目标语言类型字符串
///
/// 由各语言生成器共享使用
pub trait TypeMapper {
    fn map_type(&self, ann: &TypeAnnotation) -> String;
}
