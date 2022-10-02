use js_sys::Function;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::*, JsCast, JsValue};
use web_sys::window;

type Result<T> = std::result::Result<T, JsValue>;

pub trait CustomElementImpl: IntoWasmAbi {
    fn class_name() -> &'static str;
    fn element_name() -> &'static str;

    fn construct() -> Self;
}

pub struct WebComponentHandle<T: CustomElementImpl + 'static> {
    pub impl_handle: Closure<dyn FnMut() -> T>,
    pub element_constructor: Function,
}

pub fn define_web_component<T>() -> Result<WebComponentHandle<T>>
where
    T: CustomElementImpl + 'static,
{
    let body = format!(
        "class {name} extends HTMLElement {{
    constructor() {{
        super();
        //this.impl = impl();
    }}
}}
customElements.define(\"{element_name}\", {name});
var element = customElements.get(\"{element_name}\");
return element;",
        name = T::class_name(),
        element_name = T::element_name(),
    );
    let fun = Function::new_with_args("impl", &body);
    let f: Box<dyn FnMut() -> T> = Box::new(|| T::construct());
    // TODO(jwall): Check the lifetimes on this guy.
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

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen]
    pub struct MyElementImpl();

    impl CustomElementImpl for MyElementImpl {
        fn class_name() -> &'static str {
            "MyElement"
        }

        fn element_name() -> &'static str {
            "my-element"
        }

        fn construct() -> Self {
            Self()
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
        let fun = obj.element_constructor.dyn_ref::<Function>().unwrap();
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
