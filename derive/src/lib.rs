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
use proc_macro2::{Literal, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, AttributeArgs, Ident, ItemStruct, Lit, LitStr, Meta,
    NestedMeta, Path,
};

fn expand_crate_ref(name: &str, path: Path) -> syn::Path {
    let found_crate = crate_name(name).expect(&format!("{} is present in `Cargo.toml`", name));

    match found_crate {
        FoundCrate::Itself => parse_quote!( crate::#path ),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            parse_quote!( #ident::#path )
        }
    }
}

fn get_class_and_element_names(args: Vec<NestedMeta>) -> (Literal, Literal, Literal) {
    let mut class_name = None;
    let mut element_name = None;
    let mut observed_attributes = None;
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
            } else if nv.path.is_ident("observed_attrs") {
                if let Lit::Str(nm) = nv.lit {
                    observed_attributes = Some(nm);
                }
            }
        }
    }
    let class_name = class_name
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("", Span::call_site()).token());

    let element_name = element_name
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("", Span::call_site()).token());
    let observed_attributes = observed_attributes
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("[]", Span::call_site()).token());
    (class_name, element_name, observed_attributes)
}

fn expand_component_def(
    struct_name: &Ident,
    class_name: &Literal,
    element_name: &Literal,
) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("web-component-rs", parse_quote!(WebComponentDef));
    parse_quote! {
        impl #trait_path for #struct_name {
            fn element_name() -> &'static str {
                #element_name
            }

            fn class_name() -> &'static str {
                #class_name
            }

        }
    }
}

fn expand_struct_trait_shim(struct_name: &Ident, observed_attrs: Literal) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("web-component-rs", parse_quote!(WebComponentDef));
    let handle_path = expand_crate_ref("web-component-rs", parse_quote!(WebComponentHandle));
    parse_quote! {
        impl #struct_name {
            pub fn element_name() -> &'static str {
                <Self as #trait_path>::element_name()
            }

            pub fn class_name() -> &'static str {
                <Self as #trait_path>::class_name()
            }

            pub fn define() -> std::result::Result<#handle_path<#struct_name>, JsValue> {
                use wasm_bindgen::JsCast;
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
        console.log(this.textContent);
    }}
    
    disconnectedCallback() {{
        this._impl.disconnected_impl(this);
        console.log(this.textContent);
    }}

    static get observedAttributes() {{
        return {observed_attributes};
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
                    observed_attributes = #observed_attrs,
                );
                let fun = Function::new_with_args("impl", &body);
                let f: Box<dyn FnMut() -> Self> = Box::new(|| {
                    let obj = Self::new();
                    obj
                });
                let constructor_handle = Closure::wrap(f);
                let element = fun
                    .call1(
                        &window().unwrap(),
                        constructor_handle.as_ref().unchecked_ref::<Function>(),
                    )?
                    .dyn_into()?;
                Ok(WebComponentHandle {
                    element_constructor: element,
                    impl_handle: constructor_handle,
                })
            }
        }
    }
}

fn expand_wasm_shim(struct_name: &Ident) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("web-component-rs", parse_quote!(WebComponentBinding));
    parse_quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #struct_name {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new() -> Self {
                Self::default()
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn create() -> web_sys::Element {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element(Self::element_name())
                    .unwrap()
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn connected_impl(&self, element: &web_sys::HtmlElement) {
                use #trait_path;
                self.connected(element);
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn disconnected_impl(&self, element: &web_sys::HtmlElement) {
                use #trait_path;
                self.disconnected(element);
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn adopted_impl(&self, element: &web_sys::HtmlElement) {
                use #trait_path;
                self.adopted(element);
            }


            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn attribute_changed_impl(
                &self,
                element: &web_sys::HtmlElement,
                name: wasm_bindgen::JsValue,
                old_value: wasm_bindgen::JsValue,
                new_value: wasm_bindgen::JsValue,
            ) {
                use #trait_path;
                self.attribute_changed(element, name, old_value, new_value);
            }
        }
    }
}

fn expand_binding(struct_name: &Ident) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("web-component-rs", parse_quote!(WebComponent));
    parse_quote!(
        impl #trait_path for #struct_name {}
    )
}

fn expand_struct(
    item_struct: ItemStruct,
    class_name: Literal,
    element_name: Literal,
    observed_attributes: Literal,
) -> TokenStream {
    let struct_name = item_struct.ident.clone();
    let component_def = expand_component_def(&struct_name, &class_name, &element_name);
    let non_wasm_impl = expand_struct_trait_shim(&struct_name, observed_attributes);
    let wasm_shim = expand_wasm_shim(&struct_name);
    let binding_trait = expand_binding(&struct_name);
    let expanded = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        #[derive(Default, Debug)]
        #item_struct
        #component_def
        #non_wasm_impl
        #binding_trait
        #wasm_shim
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn web_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO(jwall): Attrs for class name and element name
    // Gather our attributes
    let args = parse_macro_input!(attr as AttributeArgs);
    let item_struct = parse_macro_input!(item as ItemStruct);

    let (class_name, element_name, observed_attributes) = get_class_and_element_names(args);

    expand_struct(item_struct, class_name, element_name, observed_attributes)
}
