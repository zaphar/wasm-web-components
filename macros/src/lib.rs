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
use inflector::Inflector;
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

fn get_class_and_element_names(
    args: Vec<NestedMeta>,
    struct_name: &Ident,
) -> (Literal, Literal, Literal, Literal) {
    let mut class_name = None;
    let mut element_name = None;
    let mut observed_attributes = None;
    let mut observed_events = None;
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
            } else if nv.path.is_ident("observed_events") {
                if let Lit::Str(nm) = nv.lit {
                    observed_events = Some(nm);
                }
            }
        }
    }

    let class_name = class_name.map(|n| n.token()).unwrap_or_else(|| {
        LitStr::new(struct_name.to_string().as_ref(), Span::call_site()).token()
    });

    let element_name = match element_name.map(|n| n.token()) {
        Some(n) => n,
        None => {
            let class_kebab = class_name.to_string().to_kebab_case().to_lowercase();
            LitStr::new(&class_kebab, Span::call_site()).token()
        }
    };

    let observed_attributes = observed_attributes
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("[]", Span::call_site()).token());
    let observed_events = observed_events
        .map(|n| n.token())
        .unwrap_or_else(|| LitStr::new("[]", Span::call_site()).token());
    (class_name, element_name, observed_attributes, observed_events)
}

fn expand_component_def(
    struct_name: &Ident,
    class_name: &Literal,
    element_name: &Literal,
) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("wasm-web-component", parse_quote!(WebComponentDef));
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

fn expand_wc_struct_trait_shim(
    struct_name: &Ident,
    once_name: &Ident,
    observed_attrs: Literal,
    observed_events: Literal,
) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("wasm-web-component", parse_quote!(WebComponentDef));
    let handle_path = expand_crate_ref("wasm-web-component", parse_quote!(WebComponentHandle));
    parse_quote! {
        impl #struct_name {
            pub fn element_name() -> &'static str {
                <Self as #trait_path>::element_name()
            }

            pub fn class_name() -> &'static str {
                <Self as #trait_path>::class_name()
            }

            #[doc = "Defines this web component element exactly once. Subsequent calls are noops."]
            pub fn define_once() {
                #once_name.call_once(|| {
                    let _ = Self::define();
                });
            }

            #[doc = "Defines this web component element if not defined already otherwise returns an error."]
            pub fn define() -> std::result::Result<#handle_path, wasm_bindgen::JsValue> {
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
        var self = this;
        self._impl = impl();
        self._impl.init();
        for (const t of self.getObservedEvents()) {{
            self.addEventListener(t, function(evt) {{ self.handleComponentEvent(evt); }} );
        }}
    }}

    connectedCallback() {{
        this._impl.connected_impl(this);
    }}
    
    disconnectedCallback() {{
        this._impl.disconnected_impl(this);
    }}

    static get observedAttributes() {{
        return {observed_attributes};
    }}

    observedEvents() {{
        return {observed_events};
    }}

    adoptedCallback() {{
        this._impl.adopted_impl(this);
    }}
    
    attributeChangedCallback(name, oldValue, newValue) {{
        this._impl.attribute_changed_impl(this, name, oldValue, newValue);
    }}

    // TODO(jwall): We need to provide a way to attach this event handler properly
    handleComponentEvent(evt) {{
        this._impl.handle_component_event_impl(this, evt);
    }}
}}
customElements.define(\"{element_name}\", {name});
var element = customElements.get(\"{element_name}\");
return element;",
                    name = Self::class_name(),
                    element_name = Self::element_name(),
                    observed_attributes = #observed_attrs,
                    observed_events = #observed_events,
                );
                let fun = js_sys::Function::new_with_args("impl", &body);
                let f: Box<dyn FnMut() -> Self> = Box::new(|| {
                    let obj = Self::new();
                    obj
                });
                let constructor_handle = wasm_bindgen::prelude::Closure::wrap(f).into_js_value().unchecked_into::<js_sys::Function>();
                let element = fun
                    .call1(
                        &window().unwrap(),
                        constructor_handle.as_ref(),
                    )?
                    .dyn_into()?;
                Ok(#handle_path {
                    element_constructor: element,
                })
            }
        }
    }
}

fn expand_wasm_shim(struct_name: &Ident) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("wasm-web-component", parse_quote!(WebComponentBinding));
    parse_quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #struct_name {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new() -> Self {
                Self::default()
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

            pub fn handle_component_event_impl(&self, element: &web_sys::HtmlElement, event: &web_sys::Event) {
                use #trait_path;
                self.handle_event(element, event);
            }
        }
    }
}

fn expand_binding(struct_name: &Ident) -> syn::ItemImpl {
    let trait_path = expand_crate_ref("wasm-web-component", parse_quote!(WebComponent));
    parse_quote!(
        impl #trait_path for #struct_name {}
    )
}

fn expand_web_component_struct(
    item_struct: ItemStruct,
    class_name: Literal,
    element_name: Literal,
    observed_attributes: Literal,
    observed_events: Literal,
) -> TokenStream {
    let struct_name = item_struct.ident.clone();
    let struct_once_name = Ident::new(
        &(struct_name.to_string().to_snake_case().to_uppercase() + "_ONCE"),
        Span::call_site(),
    );
    let component_def = expand_component_def(&struct_name, &class_name, &element_name);
    let non_wasm_impl =
        expand_wc_struct_trait_shim(&struct_name, &struct_once_name, observed_attributes, observed_events);
    let wasm_shim = expand_wasm_shim(&struct_name);
    let binding_trait = expand_binding(&struct_name);
    let expanded = quote! {
        use std::sync::Once;
        use wasm_bindgen;
        #[allow(non_snake_case)]
        static #struct_once_name: Once = Once::new();
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

#[cfg(feature = "HtmlTemplateElement")]
fn expand_template_struct(item_struct: ItemStruct) -> TokenStream {
    let struct_name = item_struct.ident.clone();
    let struct_once_name = Ident::new(
        &(struct_name.to_string().to_snake_case().to_uppercase() + "_ONCE"),
        Span::call_site(),
    );
    let trait_path = expand_crate_ref("wasm-web-component", parse_quote!(TemplateElement));
    let expanded = quote! {
        use std::sync::OnceLock;
        use web_sys::Node;
        static #struct_once_name: OnceLock<Option<String>> = OnceLock::new();
        #item_struct
        impl #trait_path for #struct_name {}
        impl #struct_name {
            #[doc = "Defines this HtmlTemplateElement and adds it to the document exactly once. Subsequent calls are noops. Returns the the template element id it exists on the template element."]
            pub fn define_once() -> Option<&'static Option<String>> {
                #struct_once_name.get_or_init(|| {
                    let template_element = Self::render();
                    let id: Option<String> = template_element.get_attribute("id");
                    let body = web_sys::window().expect("Failed to get window")
                        .document().expect("Failed to get window document").
                        body().expect("Failed to get document body");
                    body.append_child(template_element.as_ref()).expect("Failed to add template element to document");
                    return id;
                });
                return #struct_once_name.get();
            }
            
            #[doc = "Returns the the template element id it exists. None if the element has not been defined yet. Some(&None) if the element has no id. Some(&Some(id)) if the element has an id."]
            pub fn get_id() -> Option<&'static Option<String>> {
                return #struct_once_name.get();
            }
        }
    };
    TokenStream::from(expanded)
}

/// Creates the necessary Rust and Javascript shims for a Web Component.
#[proc_macro_attribute]
pub fn web_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO(jwall): Attrs for class name and element name
    // Gather our attributes
    let args = parse_macro_input!(attr as AttributeArgs);
    let item_struct = parse_macro_input!(item as ItemStruct);

    let (class_name, element_name, observed_attributes, observed_events) =
        get_class_and_element_names(args, &item_struct.ident);

    expand_web_component_struct(item_struct, class_name, element_name, observed_attributes, observed_events)
}

/// Creates the neccessary Rust and Javascript shims for rendering an HtmlTemplateElement
#[cfg(feature = "HtmlTemplateElement")]
#[proc_macro_attribute]
pub fn template_element(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    expand_template_struct(item_struct)
}
