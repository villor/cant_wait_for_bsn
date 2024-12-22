// use std::iter;

// use bevy_utils::HashMap;
// use syn::{visit::Visit, Ident, Macro, UseGlob, UseName, UsePath, UseRename};

use syn::{visit::Visit, Macro};

// #[derive(Debug, Clone)]
// enum FlatUse<'ast> {
//     Name {
//         ident: &'ast Ident,
//         path: Vec<&'ast Ident>,
//     },
//     Glob(Vec<&'ast Ident>),
// }

#[derive(Default)]
pub struct BsnMacroVisitor<'ast> {
    pub invocations: Vec<&'ast Macro>,
    // flat_use_stack: Vec<FlatUse<'ast>>,
    // use_path_stack: Vec<&'ast Ident>,
}

// impl<'ast> BsnMacroVisitor<'ast> {
//     fn open_scope(&self) -> usize {
//         self.flat_use_stack.len()
//     }

//     fn close_scope(&mut self, scope: usize) {
//         self.flat_use_stack.truncate(scope);
//     }
// }

impl<'ast> Visit<'ast> for BsnMacroVisitor<'ast> {
    // fn visit_block(&mut self, node: &'ast syn::Block) {
    //     let scope = self.open_scope();
    //     syn::visit::visit_block(self, node);
    //     self.close_scope(scope);
    // }

    // fn visit_use_path(&mut self, node: &'ast UsePath) {
    //     self.use_path_stack.push(&node.ident);
    //     syn::visit::visit_use_path(self, node);
    //     self.use_path_stack.pop();
    // }

    // fn visit_use_name(&mut self, node: &'ast UseName) {
    //     self.flat_use_stack.push(FlatUse::Name {
    //         ident: &node.ident,
    //         path: self
    //             .use_path_stack
    //             .iter()
    //             .chain(iter::once(&&node.ident))
    //             .map(|i| *i)
    //             .collect(),
    //     });
    // }

    // fn visit_use_rename(&mut self, node: &'ast UseRename) {
    //     self.flat_use_stack.push(FlatUse::Name {
    //         ident: &node.rename,
    //         path: self
    //             .use_path_stack
    //             .iter()
    //             .chain(iter::once(&&node.ident))
    //             .map(|i| *i)
    //             .collect(),
    //     });
    // }

    // fn visit_use_glob(&mut self, _: &'ast UseGlob) {
    //     self.flat_use_stack
    //         .push(FlatUse::Glob(self.use_path_stack.clone()));
    // }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.is_ident("bsn") {
            // let (named_uses, glob_uses) = self.flat_use_stack.iter().fold(
            //     (HashMap::new(), Vec::new()),
            //     |(mut named_uses, mut glob_uses), u| {
            //         match u {
            //             FlatUse::Name { ident, path } => {
            //                 named_uses.insert(
            //                     ident.to_string(),
            //                     path.iter()
            //                         .map(|i| i.to_string())
            //                         .collect::<Vec<_>>()
            //                         .join("::"),
            //                 );
            //             }
            //             FlatUse::Glob(path) => {
            //                 glob_uses.push(
            //                     path.iter()
            //                         .map(|i| i.to_string())
            //                         .collect::<Vec<_>>()
            //                         .join("::"),
            //                 );
            //             }
            //         }
            //         (named_uses, glob_uses)
            //     },
            // );
            self.invocations.push(node);
        }
        syn::visit::visit_macro(self, node);
    }
}
