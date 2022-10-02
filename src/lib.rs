use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::window;

type Result<T> = std::result::Result<T, JsValue>;

pub trait CustomElementImpl {
    fn class_name() -> &'static str;
    fn element_name() -> &'static str;
}

pub fn define_web_component<T: CustomElementImpl>() -> Result<JsValue> {
    let body = format!(
        "class {name} extends HTMLElement {{
    constructor() {{
        super();
    }}
}}
customElements.define(\"{element_name}\", {name});
var element = customElements.get(\"{element_name}\");
return element;",
        name = T::class_name(),
        element_name = T::element_name(),
    );
    let fun = Function::new_no_args(&body);
    Ok(fun.call0(&window().unwrap())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::{prelude::*, JsCast};
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    pub struct MyElementImpl();

    impl CustomElementImpl for MyElementImpl {
        fn class_name() -> &'static str {
            "MyElement"
        }

        fn element_name() -> &'static str {
            "my-element"
        }
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log(message: String);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log_with_val(message: String, val: &JsValue);
    }

    #[wasm_bindgen_test]
    fn test_component_definition() {
        let obj = define_web_component::<MyElementImpl>().expect("Failed to define web component");
        let fun = obj.dyn_ref::<Function>().unwrap();
        assert_eq!(fun.name(), MyElementImpl::class_name());

        let element = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element(MyElementImpl::element_name())
            .unwrap();
        assert_eq!(
            element.tag_name().to_uppercase(),
            MyElementImpl::element_name().to_uppercase()
        );
    }
}
