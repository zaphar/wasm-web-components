<!--
 Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
 
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 
     http://www.apache.org/licenses/LICENSE-2.0
 
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->
# An experiment in Rust Web Assembly Web Components

This set of crates should not be considered production ready. Web Assembly is not
quite ready for general use in a Web Component. Getting this to work involved 
something of a Rub Goldberg machine involving a Javascript shim and a wasm_bindgen
Rust shim with a several traits. The boilerplate is generated using a proc-macro
to make it more generally reusable.