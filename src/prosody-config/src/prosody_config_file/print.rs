// prosody-config
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::utils::def;
use super::*;

const INDENT: &'static str = "  ";

impl ToString for ProsodyConfigFile {
    fn to_string(&self) -> String {
        let mut acc = "".to_string();
        self.print(&mut acc, 0);
        acc.to_string()
    }
}

/// > NOTE: We didn't use `std::fmt::Display` because we needed to pass additional
/// >   information (identation) to the printing function.
trait Print {
    fn print(&self, acc: &mut String, indent: u8);
}

impl Print for ProsodyConfigFileSection {
    fn print(&self, acc: &mut String, indent: u8) {
        match self {
            Self::VirtualHost {
                comments,
                hostname,
                settings,
            } => {
                comments.print(acc, indent);
                acc.push_str(&format!("VirtualHost {hostname:?}\n"));
                settings.print(acc, indent + 1);
                keep_one_empty_line(acc);
            }
            Self::Component {
                comments,
                hostname,
                plugin,
                name,
                settings,
            } => {
                comments.print(acc, indent);
                acc.push_str(&format!("Component {hostname:?} {plugin:?}\n"));
                // Print name definition in a group to add an empty line after it
                Group::from(def("name", name.as_str())).print(acc, indent + 1);
                settings.print(acc, indent + 1);
                keep_one_empty_line(acc);
            }
        }
    }
}

impl Print for ProsodyConfigFile {
    fn print(&self, acc: &mut String, indent: u8) {
        self.header.print(acc, indent);

        if !self.global_settings.is_empty() {
            LuaComment::new("Base server configuration").print(acc, indent);
            self.global_settings.print(acc, indent);
        }

        if !self.additional_sections.is_empty() {
            LuaComment::new("Server hosts and components").print(acc, indent);
            self.additional_sections.print(acc, indent);
        }

        trim_enf_of_file(acc);
    }
}

// MARK: - Atoms

impl<T: Print> Print for Option<T> {
    fn print(&self, acc: &mut String, indent: u8) {
        if let Some(value) = self {
            value.print(acc, indent);
        }
    }
}

impl<T: Print> Print for Vec<T> {
    fn print(&self, acc: &mut String, indent: u8) {
        for element in self.iter() {
            element.print(acc, indent);
        }
    }
}

impl<T: Print> Print for Group<T> {
    fn print(&self, acc: &mut String, indent: u8) {
        if self.elements.is_empty() {
            return;
        }

        self.comment.print(acc, indent);
        self.elements.print(acc, indent);
        // Add an empty line at the end of a group
        acc.push('\n');
    }
}

impl Print for LuaComment {
    fn print(&self, acc: &mut String, indent: u8) {
        add_indent(acc, indent);
        acc.push_str(&format!("-- {}\n", &self.0));
    }
}

impl Print for LuaDefinition {
    fn print(&self, acc: &mut String, indent: u8) {
        self.comment.print(acc, indent);
        add_indent(acc, indent);
        acc.push_str(&self.key);
        acc.push_str(" = ");
        self.value.print(acc, indent);
        acc.push('\n');
    }
}

impl Print for LuaNumber {
    fn print(&self, acc: &mut String, indent: u8) {
        match self {
            Self::Scalar(n) => acc.push_str(&format!("{n}")),
            Self::Product(lhs, rhs) => {
                lhs.print(acc, indent);
                acc.push_str(" * ");
                rhs.print(acc, indent);
            }
        }
    }
}

impl Print for LuaValue {
    fn print(&self, acc: &mut String, indent: u8) {
        match self {
            Self::Bool(b) => acc.push_str(&format!("{b}")),
            Self::Number(n) => n.print(acc, indent),
            Self::String(s) => acc.push_str(&format!("{s:?}")),
            Self::List(list) => match list.len() {
                0 => acc.push_str("{}"),
                1 if list[0].is_scalar() => {
                    acc.push_str("{ ");
                    list[0].print(acc, indent);
                    acc.push_str(" }");
                }
                _ => {
                    acc.push_str("{\n");
                    for element in list.iter() {
                        add_indent(acc, indent + 1);
                        element.print(acc, indent + 1);
                        acc.push_str(";\n");
                    }
                    add_indent(acc, indent);
                    acc.push('}');
                }
            },
            Self::Map(map) => {
                acc.push_str("{\n");
                for (key, value) in map.iter() {
                    add_indent(acc, indent + 1);
                    acc.push_str(key);
                    acc.push_str(" = ");
                    value.print(acc, indent + 1);
                    acc.push_str(";\n");
                }
                add_indent(acc, indent);
                acc.push('}');
            }
        }
    }
}

// MARK: - Helpers

fn add_indent(acc: &mut String, n: u8) {
    for _ in 0..n {
        acc.push_str(INDENT);
    }
}

fn keep_one_empty_line(acc: &mut String) {
    *acc = acc.trim_end().to_string();
    acc.push_str("\n\n");
}

fn trim_enf_of_file(acc: &mut String) {
    *acc = acc.trim_end().to_string();
    acc.push('\n');
}
