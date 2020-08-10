#![allow(dead_code)] // TODO: don't forget to remove

use crate::descriptor::FileDescriptorProto;
use crate::reflect::file::dynamic::DynamicFileDescriptor;
use crate::reflect::find_message_or_enum::{find_message_or_enum, MessageOrEnum};
use crate::reflect::GeneratedFileDescriptor;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

pub(crate) mod dynamic;
pub(crate) mod generated;

#[derive(Clone)]
pub(crate) enum FileDescriptorImpl {
    Generated(&'static GeneratedFileDescriptor),
    Dynamic(Arc<DynamicFileDescriptor>),
}

impl PartialEq for FileDescriptorImpl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FileDescriptorImpl::Generated(a), FileDescriptorImpl::Generated(b)) => {
                *a as *const GeneratedFileDescriptor == *b as *const GeneratedFileDescriptor
            }
            (FileDescriptorImpl::Dynamic(a), FileDescriptorImpl::Dynamic(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Hash for FileDescriptorImpl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            FileDescriptorImpl::Generated(g) => {
                Hash::hash(&(*g as *const GeneratedFileDescriptor), state)
            }
            FileDescriptorImpl::Dynamic(a) => {
                Hash::hash(&(&**a as *const DynamicFileDescriptor), state)
            }
        }
    }
}

impl Eq for FileDescriptorImpl {}

/// Reflection for objects defined in `.proto` file (messages, enums, etc).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FileDescriptor {
    pub(crate) imp: FileDescriptorImpl,
}

impl FileDescriptor {
    /// This function is called from generated code, it is not stable, and should not be called.
    pub fn new_generated_2(generated: &'static GeneratedFileDescriptor) -> FileDescriptor {
        FileDescriptor {
            imp: FileDescriptorImpl::Generated(generated),
        }
    }

    /// Dynamic message created from [`FileDescriptorProto`] without generated files.
    pub fn new_dynamic(
        proto: FileDescriptorProto,
        dependencies: Vec<FileDescriptor>,
    ) -> FileDescriptor {
        FileDescriptor {
            imp: FileDescriptorImpl::Dynamic(Arc::new(DynamicFileDescriptor::new(
                proto,
                dependencies,
            ))),
        }
    }

    /// `.proto` data for this file.
    pub fn get_proto(&self) -> &FileDescriptorProto {
        match &self.imp {
            FileDescriptorImpl::Generated(g) => &g.proto,
            FileDescriptorImpl::Dynamic(d) => &d.proto,
        }
    }

    fn get_deps(&self) -> &[FileDescriptor] {
        match &self.imp {
            FileDescriptorImpl::Generated(g) => &g.dependencies,
            FileDescriptorImpl::Dynamic(d) => &d.dependencies,
        }
    }

    fn get_all_files(&self) -> Vec<&FileDescriptor> {
        let mut r = Vec::new();
        let mut visited = HashSet::new();

        let mut stack = Vec::new();
        stack.push(self);
        while let Some(file) = stack.pop() {
            if !visited.insert(file) {
                continue;
            }

            r.push(file);
            stack.extend(file.get_deps());
        }

        r
    }

    pub(crate) fn find_message_or_enum_proto_in_all_files<'a>(
        &'a self,
        full_name: &str,
    ) -> Option<(String, MessageOrEnum<'a>)> {
        // sanity check
        assert!(!full_name.starts_with("."));
        for file in self.get_all_files() {
            // sanity check
            let package = file.get_proto().get_package();
            assert!(!package.starts_with("."));

            let name_to_package = if package.is_empty() {
                full_name
            } else {
                let after_package = &full_name[package.len()..];
                if !after_package.starts_with(".") {
                    continue;
                }
                &after_package[1..]
            };

            if let Some(me) = find_message_or_enum(file.get_proto(), name_to_package) {
                return Some(me);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::descriptor;

    #[test]
    fn eq() {
        assert!(descriptor::file_descriptor() == descriptor::file_descriptor().clone());
    }
}
