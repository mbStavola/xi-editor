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

#[macro_use]
extern crate log;

use crate::tree_iter::TreeIter;
use std::{collections::HashMap, path::Path};
use tree_sitter::{InputEdit, Language, Parser, Point, Tree};
use xi_core_lib::{ConfigTable, LanguageId, ViewId};
use xi_plugin_lib::CoreProxy;
use xi_plugin_lib::{mainloop, Plugin, StateCache, View};
use xi_rope::rope::RopeDelta;
use xi_trace::{trace, trace_block, trace_payload};

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

    fn initialize(&mut self, _core: CoreProxy) {
        info!("nvujf3k3jfkdo2lk");
    }

    fn update(
        &mut self,
        view: &mut View<Self::Cache>,
        delta: Option<&RopeDelta>,
        _edit_type: String,
        _author: String,
    ) {
        info!("wefwf");
        let view_id = view.get_id();
        if let Some(view_state) = self.view_states.get_mut(&view_id) {
            view_state.process_delta(view, delta.unwrap());
        }
        view.schedule_idle();
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
        info!("fhj32ko3l42f");
        let view_id = view.get_id();
        let mut view_state = ViewState::new();

        view_state.process_document(view);
        self.view_states.insert(view_id, view_state);
    }

    fn config_changed(&mut self, _view: &mut View<Self::Cache>, _changes: &ConfigTable) {}

    fn language_changed(&mut self, view: &mut View<Self::Cache>, _old_lang: LanguageId) {
        info!("vvvvvmksm");
        let view_id = view.get_id();
        if let Some(view_state) = self.view_states.get_mut(&view_id) {
            view_state.change_language(view.get_language_id());
            view_state.process_document(view);
        }
    }

    fn idle(&mut self, view: &mut View<Self::Cache>) {
        let view_id = view.get_id();

        if let Some(_view_state) = self.view_states.get_mut(&view_id) {
            view.schedule_idle();
        }
    }
}

struct ViewState {
    parser: Parser,
    tree: Option<Tree>,
}

impl ViewState {
    fn new() -> ViewState {
        info!("fwg");
        ViewState { parser: Parser::new(), tree: None }
    }

    fn process_delta(&mut self, view: &mut View<TreeCache>, delta: &RopeDelta) {
        let (iv, new_length) = delta.summary();

        // [[[[]]]]] 5
        // 0   5   10
        // s   n   e
        // |---|---|        15
        // |-------|--------|
        // s       e        n

        self.tree = {
            info!("iiiiiiiiiiiiiiiiiiiiifffffffffff");
            let old_tree: Option<&Tree> = if let Some(ref mut tree) = self.tree {
                // TODO: How do we get the proper row & column for each point?
                if let Some(line_num) = view.get_frontier() {
                    let (_line_num, _offset, _state) = view.get_prev(line_num);

                    let _old_length = (iv.start - iv.end) as u32;
                    let new_end = iv.start - new_length;

                    let start_line = view.line_of_offset(iv.start).unwrap() as u32;

                    let old_end_line = view.line_of_offset(iv.end).unwrap() as u32;
                    let new_end_line = view.line_of_offset(new_end).unwrap() as u32;

                    tree.edit(&InputEdit {
                        start_byte: iv.start as u32,
                        old_end_byte: iv.end as u32,
                        new_end_byte: new_end as u32,
                        start_position: Point::new(start_line, iv.start as u32),
                        old_end_position: Point::new(old_end_line, iv.end as u32),
                        new_end_position: Point::new(new_end_line, new_end as u32),
                    });
                }

                Some(tree)
            } else {
                None
            };

            // TODO: We want to be able to use parse_u8 and pass in a closure that utilizes view.get_line()
            info!("fiiiiiiiiiiiiiiiiiiii");
            let input = view.get_document().unwrap();
            self.parser.parse_str(&input, old_tree)
        };
        self.highlight();
    }

    fn process_document(&mut self, view: &mut View<TreeCache>) {
        info!("cccccccccccccccccc");
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
                info!(
                    "Walk: {:?}, kind {}, kind id {}, changes {}, error {}",
                    node,
                    node.kind(),
                    node.kind_id(),
                    node.has_changes(),
                    node.has_error()
                );
                node.kind().to_string()
            })
            .collect();

        info!("Scopes: {:?}", scopes);
    }

    fn change_language(&mut self, language_id: &LanguageId) {
        info!("language: {:?}", language_id.as_ref());
        let language = unsafe {
            match language_id.as_ref() {
                //"Agda" => tree_sitter_agda(),
                //"Bourne Again Shell (bash)" => tree_sitter_bash(),
                "C" => tree_sitter_c(),
                "C#" => tree_sitter_c_sharp(),
                //"C++" => tree_sitter_cpp(),
                "CSS" => tree_sitter_css(),
                "Fluent" => tree_sitter_fluent(),
                "Go" => tree_sitter_go(),
                //"haskell" => tree_sitter_haskell(),
                //"html" => tree_sitter_html(),
                "Java" => tree_sitter_java(),
                "JavaScript" => tree_sitter_javascript(),
                "JSdoc" => tree_sitter_jsdoc(),
                "JSON" => tree_sitter_json(),
                "Julia" => tree_sitter_julia(),
                //"OCaml" => tree_sitter_ocaml(),
                //"PHP" => tree_sitter_php(),
                //"Python" => tree_sitter_python(),
                "Regular Expression" => tree_sitter_regex(),
                //"Ruby" => tree_sitter_ruby(),
                "Rust" => tree_sitter_rust(),
                "Scala" => tree_sitter_scala(),
                //"Swift" => tree_sitter_swift(),
                "TypeScript" => tree_sitter_typescript(),
                _ => {
                    info!("ct");
                    //self.parser.reset();
                    return;
                }
            }
        };

        // Handle error
        self.parser.set_language(language).unwrap();
    }
}

fn main() {
    fern::Dispatch::new()
        .chain(std::io::stdout())
        .chain(fern::log_file("/Users/mbs/Library/Logs/XiEditor/tree.log").unwrap())
        .apply()
        .unwrap();

    info!("frvg");
    let mut plugin = TreeSitterPlugin::default();
    info!("dddddddddddddd");
    mainloop(&mut plugin).unwrap();
}
