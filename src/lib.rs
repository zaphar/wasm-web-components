use js_sys::Function;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::*, JsCast, JsValue};
use web_sys::{window, Element, HtmlElement};

type Result<T> = std::result::Result<T, JsValue>;

// TODO(jwall): Trait methods can't be exported out to js yet so we'll need a wrapper object or we'll need to `Derive` this api in a prop-macro.
pub trait CustomElementImpl: IntoWasmAbi {}

pub struct WebComponentHandle<T: CustomElementImpl> {
    pub impl_handle: Closure<dyn FnMut() -> T>,
    pub element_constructor: Function,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Text;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen]
    #[derive(Default, Debug)]
    pub struct MyElementImpl {}

    impl CustomElementImpl for MyElementImpl {}

    impl MyElementImpl {
        pub fn class_name() -> &'static str {
            "MyElement"
        }

        pub fn element_name() -> &'static str {
            "my-element"
        }

        pub fn define() -> Result<WebComponentHandle<Self>> {
            let registry = window().unwrap().custom_elements();
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

    #[wasm_bindgen]
    impl MyElementImpl {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[wasm_bindgen]
        pub fn create() -> Element {
            window()
                .unwrap()
                .document()
                .unwrap()
                .create_element(Self::element_name())
                .unwrap()
        }

        #[wasm_bindgen]
        pub fn connected_impl(&self, element: &HtmlElement) {
            log("Firing connected call back".to_owned());
            let node = Text::new().unwrap();
            node.set_text_content(Some("Added a text node on connect".into()));
            element.append_child(&node).unwrap();
            log_with_val("element: ".to_owned(), element);
        }

        #[wasm_bindgen]
        pub fn disconnected_impl(&self, element: &HtmlElement) {
            log("Firing discconnected call back".to_owned());
            let node = element.first_child().unwrap();
            element.remove_child(&node).unwrap();
        }

        #[wasm_bindgen]
        pub fn adopted_impl(&self, element: &HtmlElement) {
            log("Firing adopted call back".to_owned());
            let node = Text::new().unwrap();
            node.set_text_content(Some("Added a text node on adopt".into()));
            element.append_child(&node).unwrap();
            log_with_val("element: ".to_owned(), element);
        }

        pub fn observed_attributes() -> js_sys::Array {
            let attrs = js_sys::Array::new();
            attrs.push(&JsValue::from_str("class"));
            attrs
        }

        #[wasm_bindgen]
        pub fn attribute_changed_impl(
            &self,
            element: &HtmlElement,
            name: JsValue,
            old_value: JsValue,
            new_value: JsValue,
        ) {
            log("Firing attribute changed callback".to_owned());
            let node = element.first_child().unwrap();
            node.set_text_content(Some(&format!(
                "Setting {} from {} to {}",
                name.as_string().unwrap_or("None".to_owned()),
                old_value.as_string().unwrap_or("None".to_owned()),
                new_value.as_string().unwrap_or("None".to_owned()),
            )));
            element.append_child(&node).unwrap();
            log_with_val("element: ".to_owned(), element);
        }
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log(message: String);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log_with_val(message: String, val: &JsValue);
    }

    // NOTE(jwall): We can only construct the web component once and since the lifetime of the component internals is tied
    // to the handle we run this all in one single function.
    #[wasm_bindgen_test]
    fn test_component() {
        let obj = MyElementImpl::define().expect("Failed to define web component");
        let fun = obj.element_constructor.dyn_ref::<Function>().unwrap();
        assert_eq!(fun.name(), MyElementImpl::class_name());
        let element = MyElementImpl::create();
        assert_eq!(
            element.tag_name().to_uppercase(),
            MyElementImpl::element_name().to_uppercase()
        );
        let document = window().unwrap().document().unwrap();
        let body = document.body().unwrap();

        // Test the connected callback
        let node = body.append_child(element.as_ref()).unwrap();
        assert_eq!(
            element.text_content().unwrap(),
            "Added a text node on connect"
        );

        // Test the disconnected callback
        body.remove_child(&node).unwrap();
        assert_eq!(element.text_content().unwrap(), "");

        body.append_child(element.as_ref()).unwrap();
        element.set_attribute("class", "foo").unwrap();
        assert_eq!(
            element.text_content().unwrap(),
            "Setting class from None to foo"
        );

        // Test the adopted callback
        // First we need a new window with a new document to perform the adoption with.
        let new_window = window().unwrap().open().unwrap().unwrap();
        // Then we can have the new document adopt this node.
        new_window.document().unwrap().adopt_node(&element).unwrap();
        assert_eq!(
            element.text_content().unwrap(),
            "Added a text node on adopt"
        );
    }
}
