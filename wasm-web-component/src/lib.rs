use js_sys::Function;
use wasm_bindgen::JsCast;
use wasm_bindgen::{convert::IntoWasmAbi, JsValue};
#[cfg(feature = "HtmlTemplateElement")]
use web_sys::HtmlTemplateElement;
use web_sys::{window, Element, Event, HtmlElement, Window};

/// This attribute proc-macro will generate the following trait implementations
/// * [WebComponentDef](trait@WebComponentDef)
/// * [WebComponent](trait@WebComponent)
///
/// It will also generate a wasm_bindgen compatible impl block for your struct.
///
/// It expects you to implement the [WebComponentBinding](trait@WebComponentBinding)
/// trait in order to implement the callbacks.
///
/// It supports three optional attributes `name = value` parameters.
/// * `class_name = "ClassName"` - The class name to use for the javascript shim. If not provided uses the structs name instead.
/// * `element_name = "class-name"` - A valid custom element name to use for the element. if not proviced derives it from the class name.
/// * `observed_attrs = "['attr1', 'attr2']"` - A javascript array with a list of observed attributes for this compoment. Defaults to "[]".
/// * `observed_events = "['click', 'change']"` - A javascript array with a list of observed event types for this compoment. Defaults to "[]".
/// * `base_class = "HTMLInputElement"` - The HTMLElement base class this custom-element should
/// inherit from. Defaults to "HTMLElement".
///
/// It will also create a `Self::define_once` method that will define the WebComponent exactly
/// once.
///
/// ## Example
///
/// ```ignore
/// use web_sys::*;
/// use wasm_bindgen::*;
/// use wasm_web_component::{web_component, WebComponent, WebComponentHandle, WebComponentDef, WebComponentBinding};
/// 
/// #[web_component(
///     class_name = "MyElement",
///     element_name = "my-element",
///     observed_attrs = "['class']",
///     observed_events = "['click']",
///     base_class = "HTMLElement"
/// )]
/// pub struct MyElementImpl {}
/// 
/// impl WebComponentBinding for MyElementImpl {
///     fn connected(&self, element: &HtmlElement) {
///         let node = Text::new().unwrap();
///         node.set_text_content(Some("Added a text node on connect".into()));
///         element.append_child(&node).unwrap();
///     }
/// 
///     fn disconnected(&self, element: &HtmlElement) {
///         let node = element.first_child().unwrap();
///         element.remove_child(&node).unwrap();
///     }
/// 
///     fn adopted(&self, element: &HtmlElement) {
///         let node = Text::new().unwrap();
///         node.set_text_content(Some("Added a text node on adopt".into()));
///         element.append_child(&node).unwrap();
///     }
/// 
///     fn attribute_changed(
///         &self,
///         element: &HtmlElement,
///         name: JsValue,
///         old_value: JsValue,
///         new_value: JsValue,
///     ) {
///         let node = element.first_child().unwrap();
///         node.set_text_content(Some(&format!(
///             "Setting {} from {} to {}",
///             name.as_string().unwrap_or("None".to_owned()),
///             old_value.as_string().unwrap_or("None".to_owned()),
///             new_value.as_string().unwrap_or("None".to_owned()),
///         )));
///         element.append_child(&node).unwrap();
///     }
///
///     fn handle_event(&self, element: &HtmlElement, event: &Event) {
///         // handle this event
///     }
/// }
///
/// pub fn define_me() {
///    MyElementImpl::define_once();
/// }
/// ```
/// Reference [MDN Web Components Guide](https://developer.mozilla.org/en-US/docs/Web/Web_Components)
pub use wasm_web_component_macros::web_component;

/// This attribute proc-macro will generate the following trait implementation
/// [TemplateElement](trait@TemplateElement)
///
/// It will also generate a wasm_bindgen compatible impl block for your struct. It expects
/// you to implement [TemplateElementRender](trait@TemplateElementRender) trait in order to
/// allow it to implement the methods using methods from that trait.
///
/// You can define the template element exactly once by calling the `Self::define_once` method.
/// Subsequent calls to that method will be a noop. It returns one of the following values:
/// * `Some(None)` If the template doesn't have an id.
/// * `Some(Some(id))` If the template has an id.
/// * `None` Should never get returned.
///
/// A `get_id` method will also get defined for you that returns the same values with the difference that
/// if the template has not been defined yet `None` will get returned.
///
/// ## Example usage
/// ```ignore
/// use wasm_web_component::*;
/// use wasm_bindgen::*;
/// # #[cfg(feature = "HtmlTemplateElement")]
/// #[template_element]
/// pub struct MyTemplate ();
/// impl TemplateElementRender for MyTemplate {
///     fn render() -> HtmlTemplateElement {
///        let val: JsValue = window()
///            .unwrap()
///            .document()
///            .unwrap()
///            .create_element("template")
///            .unwrap()
///            .into();
///        let el: HtmlTemplateElement = val.into();
///        el.set_attribute("id", "template-id").unwrap();
///        return el;
///     }
/// }
///
/// pub fn define_it() {
///     let id: Option<&'static Option<String>> = MyTemplate::define_once();
/// }
/// ```
pub use wasm_web_component_macros::template_element;

/// Helper trait for Rust Web Components. This is autogenerated
/// by the [`#[web_component]`](web_component) attribute.
pub trait WebComponentDef: IntoWasmAbi + Default {
    fn new() -> Self {
        Self::default()
    }

    fn create() -> Element {
        Self::create_in_window(window().expect("Failed to get window"))
    }

    fn create_in_window(window: Window) -> Element {
        window
            .document()
            .expect("Failed to get document")
            .create_element(Self::element_name())
            .expect("Failed to create element")
    }

    /// Creates a custom event
    fn custom_event(event_type: &str) -> web_sys::Event {
        web_sys::CustomEvent::new(event_type).unwrap().dyn_into().unwrap()
    }

    fn element_name() -> &'static str;
    fn class_name() -> &'static str;
}

/// Trait defining the lifecycle callbacks for a Custom Element.
/// Each method is optional. You only need to implement the ones
/// you want to specify behavior for.
pub trait WebComponentBinding: WebComponentDef {
    /// Called during element construction.
    fn init(&self, _element: &HtmlElement) {
        // noop
    }
    
    fn init_mut(&mut self, _element: &HtmlElement) {
        // noop
    }
    
    /// Called when the web component is connected to the DOM.
    /// This is when you should do any setup like attaching a ShadowDom
    /// or appending elements.
    fn connected(&self, _element: &HtmlElement) {
        // noop
    }

    /// Called when the web component is connected to the DOM.
    /// This is when you should do any setup like attaching a ShadowDom
    /// or appending elements.
    fn connected_mut(&mut self, _element: &HtmlElement) {
        // noop
    }

    /// Called when the web component is disconnected from the DOM.
    fn disconnected(&self, _element: &HtmlElement) {
        // noop
    }

    /// Called when the web component is disconnected from the DOM.
    fn disconnected_mut(&mut self, _element: &HtmlElement) {
        // noop
    }

    /// Called When the web component is moved to a new document.
    fn adopted(&self, _element: &HtmlElement) {
        // noop
    }

    /// Called When the web component is moved to a new document.
    fn adopted_mut(&mut self, _element: &HtmlElement) {
        // noop
    }

    /// Called when one of the observed attributes has changed.
    /// the observedc attributes are listed in the observed_attrs argument to the
    /// `#[web_component(observed_attrs = "['attr1', 'attr2']")` attribute.
    fn attribute_changed(
        &self,
        _element: &HtmlElement,
        _name: JsValue,
        _old_value: JsValue,
        _new_value: JsValue,
    ) {
        // noop
    }

    /// Called when one of the observed attributes has changed.
    /// the observedc attributes are listed in the observed_attrs argument to the
    /// `#[web_component(observed_attrs = "['attr1', 'attr2']")` attribute.
    fn attribute_changed_mut(
        &mut self,
        _element: &HtmlElement,
        _name: JsValue,
        _old_value: JsValue,
        _new_value: JsValue,
    ) {
        // noop
    }

    /// Top level event handler for this custom element.
    fn handle_event(&self, _element: &HtmlElement, _event: &Event) {
        // noop
    }
    
    /// Top level event handler for this custom element.
    fn handle_event_mut(&mut self, _element: &HtmlElement, _event: &Event) {
        // noop
    }
}

/// Marker trait used in the generated shims to assert that there are Rust implemtntations
/// of the callback functions for the component.
pub trait WebComponent: WebComponentBinding {}

/// Defines the template element rendering method.
#[cfg(feature = "HtmlTemplateElement")]
pub trait TemplateElementRender {
    // Creates and returns an HtmlTemplateElement.
    fn render() -> HtmlTemplateElement;
}

/// Marker trait used in the generated shims to assert that there are Rust implemtntations
/// of the rendering function for the component.
#[cfg(feature = "HtmlTemplateElement")]
pub trait TemplateElement: TemplateElementRender {}

/// A handle for your WebComponent Definition. Offers easy access to construct your
/// element.
pub struct WebComponentHandle {
    /// A javascript function that can construct your element.
    pub element_constructor: Function,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Text;
    use web_sys::{console, window, HtmlElement};

    use wasm_web_component_macros::web_component;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log(message: String);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log_with_val(message: String, val: &JsValue);
    }

    pub struct Timer<'a> {
        name: &'a str,
    }

    impl<'a> Timer<'a> {
        pub fn new(name: &'a str) -> Timer<'a> {
            console::time_with_label(name);
            Timer { name }
        }
    }

    impl<'a> Drop for Timer<'a> {
        fn drop(&mut self) {
            console::time_end_with_label(self.name);
        }
    }

    #[wasm_bindgen_test]
    pub fn bench_mark_elements() {
        #[web_component(observed_attrs = "['class']")]
        pub struct BenchElement {}

        impl WebComponentBinding for BenchElement {
            fn connected(&self, element: &HtmlElement) {
                let node = Text::new().unwrap();
                node.set_text_content(Some("Added a text node on connect".into()));
                element.append_child(&node).unwrap();
            }

            fn disconnected(&self, element: &HtmlElement) {
                let node = element.first_child().unwrap();
                element.remove_child(&node).unwrap();
            }

            fn adopted(&self, element: &HtmlElement) {
                let node = Text::new().unwrap();
                node.set_text_content(Some("Added a text node on adopt".into()));
                element.append_child(&node).unwrap();
            }

            fn attribute_changed(
                &self,
                element: &HtmlElement,
                name: JsValue,
                old_value: JsValue,
                new_value: JsValue,
            ) {
                let node = element.first_child().unwrap();
                node.set_text_content(Some(&format!(
                    "Setting {} from {} to {}",
                    name.as_string().unwrap_or("None".to_owned()),
                    old_value.as_string().unwrap_or("None".to_owned()),
                    new_value.as_string().unwrap_or("None".to_owned()),
                )));
                element.append_child(&node).unwrap();
            }
        }

        Timer::new("custom-element::timing");
        let _ = BenchElement::define();

        let body = window().unwrap().document().unwrap().body().unwrap();
        for _ in 1..100000 {
            let el = BenchElement::create();
            body.append_child(&el).unwrap();
            el.set_attribute("class", "foo").unwrap();
            body.remove_child(&el).unwrap();
        }
    }

    // NOTE(jwall): We can only construct the web component once and since the lifetime of the component internals is tied
    // to the handle we run this all in one single function.
    #[wasm_bindgen_test]
    fn test_component() {
        #[web_component(
            class_name = "MyElement",
            element_name = "my-element",
            observed_attrs = "['class']",
        )]
        pub struct MyElementImpl {}

        impl WebComponentBinding for MyElementImpl {
            fn connected(&self, element: &HtmlElement) {
                let node = Text::new().unwrap();
                node.set_text_content(Some("Added a text node on connect".into()));
                element.append_child(&node).unwrap();
            }

            fn disconnected(&self, element: &HtmlElement) {
                let node = element.first_child().unwrap();
                element.remove_child(&node).unwrap();
            }

            fn adopted(&self, element: &HtmlElement) {
                let node = Text::new().unwrap();
                node.set_text_content(Some("Added a text node on adopt".into()));
                element.append_child(&node).unwrap();
            }

            fn attribute_changed(
                &self,
                element: &HtmlElement,
                name: JsValue,
                old_value: JsValue,
                new_value: JsValue,
            ) {
                let node = element.first_child().unwrap();
                node.set_text_content(Some(&format!(
                    "Setting {} from {} to {}",
                    name.as_string().unwrap_or("None".to_owned()),
                    old_value.as_string().unwrap_or("None".to_owned()),
                    new_value.as_string().unwrap_or("None".to_owned()),
                )));
                element.append_child(&node).unwrap();
            }
        }
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

        // NOTE(jwall): If we are running headless then this can fail sometimes.
        //   We don't fail the test when that happens.
        if let Ok(Some(new_window)) = window().unwrap().open() {
            // Test the adopted callback
            // First we need a new window with a new document to perform the adoption with.
            new_window.document().unwrap().adopt_node(&element).unwrap();
            assert_eq!(
                element.text_content().unwrap(),
                "Added a text node on adopt"
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_component_mut() {
        #[web_component(
            class_name = "MyElementMut",
            element_name = "my-element-mut",
            observed_attrs = "['class']",
        )]
        pub struct MyElementMutImpl {}

        impl WebComponentBinding for MyElementMutImpl {
            fn connected_mut(&mut self, element: &HtmlElement) {
                let node = Text::new().unwrap();
                node.set_text_content(Some("Added a text node on connect".into()));
                element.append_child(&node).unwrap();
            }

            fn disconnected_mut(&mut self, element: &HtmlElement) {
                let node = element.first_child().unwrap();
                element.remove_child(&node).unwrap();
            }

            fn adopted_mut(&mut self, element: &HtmlElement) {
                let node = Text::new().unwrap();
                node.set_text_content(Some("Added a text node on adopt".into()));
                element.append_child(&node).unwrap();
            }

            fn attribute_changed_mut(
                &mut self,
                element: &HtmlElement,
                name: JsValue,
                old_value: JsValue,
                new_value: JsValue,
            ) {
                let node = element.first_child().unwrap();
                node.set_text_content(Some(&format!(
                    "Setting {} from {} to {}",
                    name.as_string().unwrap_or("None".to_owned()),
                    old_value.as_string().unwrap_or("None".to_owned()),
                    new_value.as_string().unwrap_or("None".to_owned()),
                )));
                element.append_child(&node).unwrap();
            }
        }
        let obj = MyElementMutImpl::define().expect("Failed to define web component");
        let fun = obj.element_constructor.dyn_ref::<Function>().unwrap();
        assert_eq!(fun.name(), MyElementMutImpl::class_name());
        let element = MyElementMutImpl::create();
        assert_eq!(
            element.tag_name().to_uppercase(),
            MyElementMutImpl::element_name().to_uppercase()
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

        // NOTE(jwall): If we are running headless then this can fail sometimes.
        //   We don't fail the test when that happens.
        if let Ok(Some(new_window)) = window().unwrap().open() {
            // Test the adopted callback
            // First we need a new window with a new document to perform the adoption with.
            new_window.document().unwrap().adopt_node(&element).unwrap();
            assert_eq!(
                element.text_content().unwrap(),
                "Added a text node on adopt"
            );
        } else {
            assert!(false);
        }
    }
    
    #[wasm_bindgen_test]
    fn test_component_no_element_name() {
        #[web_component(class_name = "AnElement")]
        pub struct AnElement {}
        impl WebComponentBinding for AnElement {}

        assert_eq!(AnElement::element_name(), "an-element");
    }

    #[wasm_bindgen_test]
    fn test_component_no_class_name() {
        #[web_component]
        pub struct AnotherElement {}
        impl WebComponentBinding for AnotherElement {}

        assert_eq!(AnotherElement::class_name(), "AnotherElement");
        assert_eq!(AnotherElement::element_name(), "another-element");
    }

    #[wasm_bindgen_test]
    fn test_component_no_class_name_with_element_name() {
        #[web_component(element_name = "this-old-element")]
        pub struct ThisElement {}
        impl WebComponentBinding for ThisElement {}

        assert_eq!(ThisElement::class_name(), "ThisElement");
        assert_eq!(ThisElement::element_name(), "this-old-element");
    }

    // TODO(jwall): Tests for event handling

    // TODO(jwall): Benchmarks for TemplateElements?
    #[cfg(feature = "HtmlTemplateElement")]
    #[wasm_bindgen_test]
    fn test_template_element_render_once() {
        use wasm_web_component_macros::template_element;

        #[template_element]
        pub struct MyTemplate();
        impl TemplateElementRender for MyTemplate {
            fn render() -> HtmlTemplateElement {
                let val: JsValue = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("template")
                    .unwrap()
                    .into();
                let el: HtmlTemplateElement = val.into();
                el.set_attribute("id", "template-id").unwrap();
                return el;
            }
        }

        let body = window().unwrap().document().unwrap().body().unwrap();
        assert!(!body.last_child().unwrap().has_type::<HtmlTemplateElement>());
        let id = MyTemplate::define_once();
        assert_eq!(id.unwrap(), &Some(String::from("template-id")));
        assert!(body.last_child().unwrap().has_type::<HtmlTemplateElement>());
    }
}
