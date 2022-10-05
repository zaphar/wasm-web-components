// Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
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

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemStruct, Lit, LitStr, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn web_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO(jwall): Attrs for class name and element name
    // Gather our attributes
    let args = parse_macro_input!(attr as AttributeArgs);
    let mut class_name = None;
    let mut element_name = None;
    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
            if nv.path.is_ident("class_name") {
                if let Lit::Str(nm) = nv.lit {
                    class_name = Some(nm);
                }
            } else if nv.path.is_ident("element_name") {
                if let Lit::Str(nm) = nv.lit {
                    element_name = Some(nm);
                }
            }
        }
    }
    let element_name = element_name
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("", Span::call_site()).token());
    let class_name = class_name
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("", Span::call_site()).token());
    let item_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = item_struct.ident.clone();
    let expanded = quote! {
        #[wasm_bindgen]
        #item_struct

        impl #struct_name {
            pub fn element_name() -> &'static str {
                #element_name
            }

            pub fn class_name() -> &'static str {
                #class_name
            }

            pub fn define() -> Result<WebComponentHandle<#struct_name>> {
                use js_sys::Function;
                use web_sys::{window, Element, HtmlElement};
                let registry = web_sys::window().unwrap().custom_elements();
                let maybe_element = registry.get(Self::element_name());
                if maybe_element.is_truthy() {
                    return Err("Custom Element has already been defined".into());
                }
                let body = format!(
                "class {name} extends HTMLElement {{
    constructor() {{
        super();
        this._impl = impl();
    }}

    connectedCallback() {{
        this._impl.connected_impl(this);
    }}
    
    disconnectedCallback() {{
        this._impl.disconnected_impl(this);
    }}

    static get observedAttributes() {{
        console.log('observed attributes: ', attrs);
        return attrs;
    }}

    adoptedCallback() {{
        console.log('In adoptedCallback');
        this._impl.adopted_impl(this);
    }}
    
   attributeChangedCallback(name, oldValue, newValue) {{
        this._impl.attribute_changed_impl(this, name, oldValue, newValue);
    }}
}}
customElements.define(\"{element_name}\", {name});
var element = customElements.get(\"{element_name}\");
return element;",
                    name = Self::class_name(),
                    element_name = Self::element_name(),
                );
                let fun = Function::new_with_args("impl, attrs", &body);
                let f: Box<dyn FnMut() -> Self> = Box::new(|| {
                    let obj = Self::new();
                    obj
                });
                let constructor_handle = Closure::wrap(f);
                let element = fun
                    .call2(
                        &window().unwrap(),
                        constructor_handle.as_ref().unchecked_ref::<Function>(),
                        &Self::observed_attributes(),
                    )?
                    .dyn_into()?;
                Ok(WebComponentHandle {
                    element_constructor: element,
                    impl_handle: constructor_handle,
                })
            }
        }
    };

    TokenStream::from(expanded)
}
