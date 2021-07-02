// Copyright © 2021-present Derek Thurn

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Entry point to the magical world of Oak

#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::integer_division)]
#![deny(clippy::let_underscore_must_use)]
#![deny(clippy::multiple_inherent_impl)]
#![deny(clippy::str_to_string)]
#![deny(clippy::string_to_string)]
#![deny(clippy::unnecessary_self_imports)]
#![deny(clippy::unwrap_in_result)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::cloned_instead_of_copied)]
#![deny(clippy::copy_iterator)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::ref_option_ref)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::let_underscore_drop)]
#![deny(clippy::manual_ok_or)]
#![deny(clippy::map_flatten)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::needless_continue)]
#![deny(clippy::needless_for_each)]
#![deny(clippy::option_if_let_else)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnested_or_patterns)]
#![deny(clippy::unused_self)]
#![deny(clippy::used_underscore_binding)]
#![deny(clippy::wildcard_imports)]
#![deny(clippy::inconsistent_struct_constructor)]
#![deny(clippy::use_self)]
#![deny(clippy::useless_let_if_seq)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod agents;
mod game;
mod interface;
mod model;

fn main() {
    yew::start_app::<interface::main::Oak>();
}
