use js_sys::Function;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::Closure, JsValue};
use web_sys::{window, Element, HtmlElement};

pub mod macros;

pub trait WebComponentDef: IntoWasmAbi + Default {
    fn new() -> Self {
        Self::default()
    }

    fn create() -> Element {
        window()
            .unwrap()
            .document()
            .unwrap()
            .create_element(Self::element_name())
            .unwrap()
    }

    fn element_name() -> &'static str;
    fn class_name() -> &'static str;
}

pub trait WebComponentBinding: WebComponentDef {
    fn connected(&self, _element: &HtmlElement) {
        // noop
    }

    fn disconnected(&self, _element: &HtmlElement) {
        // noop
    }

    fn adopted(&self, _element: &HtmlElement) {
        // noop
    }

    fn attribute_changed(
        &self,
        _element: &HtmlElement,
        _name: JsValue,
        _old_value: JsValue,
        _new_value: JsValue,
    ) {
        // noop
    }
}

pub trait WebComponent: WebComponentBinding {}

// TODO(jwall): Trait methods can't be exported out to js yet so we'll need a wrapper object or we'll need to `Derive` this api in a prop-macro.
pub struct WebComponentHandle<T> {
    pub impl_handle: Closure<dyn FnMut() -> T>,
    pub element_constructor: Function,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Text;
    use web_sys::{window, HtmlElement};

    use web_component_derive::web_component;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[web_component(
        class_name = "MyElement",
        element_name = "my-element",
        observed_attrs = "['class']"
    )]
    pub struct MyElementImpl {}

    impl WebComponentBinding for MyElementImpl {
        fn connected(&self, element: &HtmlElement) {
            log("Firing connected call back".to_owned());
            let node = Text::new().unwrap();
            node.set_text_content(Some("Added a text node on connect".into()));
            element.append_child(&node).unwrap();
            log(format!(
                "element contents: {}",
                &element.text_content().unwrap()
            ));
        }

        fn disconnected(&self, element: &HtmlElement) {
            log("Firing discconnected call back".to_owned());
            let node = element.first_child().unwrap();
            element.remove_child(&node).unwrap();
        }

        fn adopted(&self, element: &HtmlElement) {
            log("Firing adopted call back".to_owned());
            let node = Text::new().unwrap();
            node.set_text_content(Some("Added a text node on adopt".into()));
            element.append_child(&node).unwrap();
            log_with_val("element: ".to_owned(), element);
        }

        fn attribute_changed(
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
        body.append_child(&element).unwrap();
        assert_eq!(
            element.text_content().unwrap(),
            "Added a text node on connect"
        );

        // Test the disconnected callback
        body.remove_child(&element).unwrap();
        assert_eq!(element.text_content().unwrap(), "");

        body.append_child(&element).unwrap();
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
