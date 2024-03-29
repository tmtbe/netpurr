use std::collections::BTreeSet;

use egui_code_editor::Syntax;

pub fn js_syntax() -> Syntax {
    Syntax {
        language: "JavaScript",
        case_sensitive: true,
        comment: "//",
        comment_multiline: ["/*", "*/"],
        hyperlinks: Default::default(),
        keywords: BTreeSet::from([
            // ES3 Keywords
            "break",
            "case",
            "catch",
            "continue",
            "default",
            "delete",
            "do",
            "else",
            "finally",
            "for",
            "function",
            "if",
            "in",
            "instanceof",
            "new",
            "return",
            "switch",
            "this",
            "throw",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            // ES5 Keywords
            "break",
            "case",
            "catch",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "finally",
            "for",
            "function",
            "if",
            "in",
            "instanceof",
            "new",
            "return",
            "switch",
            "this",
            "throw",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            // ES6/ES2015 Keywords
            "class",
            "const",
            "export",
            "extends",
            "import",
            "super",
            "let",
            "yield",
            "async",
            "await",
            // Reserved Words for Future Use
            "enum",
        ]),
        types: BTreeSet::from([
            "undefined",
            "null",
            "boolean",
            "number",
            "string",
            "object",
            "function",
            "symbol",
        ]),
        special: BTreeSet::from(["Self", "static", "true", "false"]),
    }
}

pub fn log_syntax() -> Syntax {
    Syntax {
        language: "Log",
        case_sensitive: false,
        comment: "//",
        comment_multiline: ["/*", "*/"],
        hyperlinks: Default::default(),
        keywords: BTreeSet::from(["log", "Info", "Warn", "Error"]),
        types: BTreeSet::from([]),
        special: BTreeSet::from([]),
    }
}
