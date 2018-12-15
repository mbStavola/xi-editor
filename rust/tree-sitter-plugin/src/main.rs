// Copyright 2016 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate tree_sitter;
extern crate xi_core_lib as xi_core;
extern crate xi_plugin_lib;
extern crate xi_rope;

use std::{collections::HashMap, path::Path};

use xi_core::{ConfigTable, LanguageId, ViewId};
use xi_plugin_lib::{mainloop, Plugin, StateCache, View};
use xi_rope::rope::RopeDelta;

use tree_sitter::{InputEdit, Language, Parser, Point, Tree};

use tree_iter::TreeIter;

mod tree_iter;

extern "C" {
    fn tree_sitter_agda() -> Language;
    fn tree_sitter_bash() -> Language;
    fn tree_sitter_c() -> Language;
    fn tree_sitter_c_sharp() -> Language;
    fn tree_sitter_cpp() -> Language;
    fn tree_sitter_css() -> Language;
    fn tree_sitter_fluent() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_haskell() -> Language;
    fn tree_sitter_html() -> Language;
    fn tree_sitter_java() -> Language;
    fn tree_sitter_javascript() -> Language;
    fn tree_sitter_jsdoc() -> Language;
    fn tree_sitter_json() -> Language;
    fn tree_sitter_julia() -> Language;
    fn tree_sitter_ocaml() -> Language;
    fn tree_sitter_php() -> Language;
    fn tree_sitter_python() -> Language;
    fn tree_sitter_regex() -> Language;
    fn tree_sitter_ruby() -> Language;
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_scala() -> Language;
    fn tree_sitter_swift() -> Language;
    fn tree_sitter_typescript() -> Language;
}

type TreeCache = StateCache<()>;

#[derive(Default)]
struct TreeSitterPlugin {
    view_states: HashMap<ViewId, ViewState>,
}

impl Plugin for TreeSitterPlugin {
    type Cache = TreeCache;

    fn update(
        &mut self,
        view: &mut View<Self::Cache>,
        delta: Option<&RopeDelta>,
        _edit_type: String,
        _author: String,
    ) {
        let view_id = view.get_id();
        if let Some(view_state) = self.view_states.get_mut(&view_id) {
            view_state.process_delta(view, delta.unwrap());
        }
    }

    fn did_save(&mut self, view: &mut View<Self::Cache>, _old: Option<&Path>) {
        let view_id = view.get_id();
        if let Some(view_state) = self.view_states.get_mut(&view_id) {
            view_state.process_document(view);
        }
    }

    fn did_close(&mut self, view: &View<Self::Cache>) {
        let view_id = view.get_id();
        self.view_states.remove(&view_id);
    }

    fn new_view(&mut self, view: &mut View<Self::Cache>) {
        let view_id = view.get_id();
        let mut view_state = ViewState::new();

        view_state.process_document(view);
        self.view_states.insert(view_id, view_state);
    }

    fn config_changed(&mut self, _view: &mut View<Self::Cache>, _changes: &ConfigTable) {}

    fn language_changed(&mut self, view: &mut View<Self::Cache>, old_lang: LanguageId) {
        let view_id = view.get_id();
        if let Some(view_state) = self.view_states.get_mut(&view_id) {
            view_state.change_language(view.get_language_id());
            view_state.process_document(view);
        }
    }
}

struct ViewState {
    parser: Parser,
    tree: Option<Tree>,
}

impl ViewState {
    fn new() -> ViewState {
        ViewState { parser: Parser::new(), tree: None }
    }

    fn process_delta(&mut self, view: &mut View<TreeCache>, delta: &RopeDelta) {
        let (iv, new_length) = delta.summary();

        self.tree = {
            let old_tree: Option<&Tree> = if let Some(ref mut tree) = self.tree {
                // TODO: How do we get the proper row & column for each point?
                if let Some(line_num) = view.get_frontier() {
                    //                    let (line_num, offset, state) = view.get_prev(line_num);
                    //
                    //                    let start_line = view.line_of_offset(iv.start).unwrap();
                    //                    let end_line = view.line_of_offset(iv.end).unwrap();
                    //
                    //                    tree.edit(&InputEdit {
                    //                        start_byte: iv.start as u32,
                    //                        old_end_byte: iv.end as u32,
                    //                        new_end_byte: (iv.start as u32) + (new_length as u32),
                    //                        start_position: Point::new(start_line as u32, iv.start as u32),
                    //                        old_end_position: Point::new(end_line as u32, iv.end as u32),
                    //                        new_end_position: Point::new(line_num as u32, iv.end as u32),
                    //                    });
                }

                Some(tree)
            } else {
                None
            };

            // TODO: We want to be able to use parse_u8 and pass in a closure that utilizes view.get_line()
            let input = view.get_document().unwrap();
            self.parser.parse_str(&input, old_tree)
        };
        self.highlight();
    }

    fn process_document(&mut self, view: &mut View<TreeCache>) {
        let input = view.get_document().unwrap();
        self.tree = self.parser.parse_str(&input, self.tree.as_ref());
        self.highlight();
    }

    fn highlight(&self) {
        // Only send over scopes for nodes that have changes
        let scopes: Vec<String> = self
            .tree
            .as_ref()
            .map(TreeIter::new)
            .into_iter()
            .flat_map(|tree_iter| tree_iter)
            .filter(|node| node.has_changes())
            .map(|node| {
                println!(
                    "Walk: {:?}, kind {}, kind id {}, changes {}, error {}",
                    node,
                    node.kind(),
                    node.kind_id(),
                    node.has_changes(),
                    node.has_error()
                );
                node.kind().to_string()
            }).collect();

        println!("Scopes: {:?}", scopes);
    }

    fn change_language(&mut self, language_id: &LanguageId) {
        let language = unsafe {
            match language_id.as_ref() {
                //"agda" => tree_sitter_agda(),
                //"bash" => tree_sitter_bash(),
                "c" => tree_sitter_c(),
                "c_sharp" => tree_sitter_c_sharp(),
                //"cpp" => tree_sitter_cpp(),
                "css" => tree_sitter_css(),
                "fluent" => tree_sitter_fluent(),
                "go" => tree_sitter_go(),
                //"haskell" => tree_sitter_haskell(),
                //"html" => tree_sitter_html(),
                "java" => tree_sitter_java(),
                "javascript" => tree_sitter_javascript(),
                "jsdoc" => tree_sitter_jsdoc(),
                "json" => tree_sitter_json(),
                "julia" => tree_sitter_julia(),
                //"ocaml" => tree_sitter_ocaml(),
                //"php" => tree_sitter_php(),
                //"python" => tree_sitter_python(),
                "regex" => tree_sitter_regex(),
                //"ruby" => tree_sitter_ruby(),
                "rust" => tree_sitter_rust(),
                "scala" => tree_sitter_scala(),
                //"swift" => tree_sitter_swift(),
                "typescript" => tree_sitter_typescript(),
                _ => {
                    self.parser.reset();
                    return;
                }
            }
        };

        // Handle error
        self.parser.set_language(language).unwrap();
    }
}

fn main() {
    let mut plugin = TreeSitterPlugin::default();
    mainloop(&mut plugin).unwrap();
}
