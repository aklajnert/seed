use crate::util::document;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
use wasm_bindgen::JsCast;

/// Attaches given `ElRef` to the DOM element.
///
/// See `ElRef` for more info.
pub fn el_ref<E: Clone>(reference: &ElRef<E>) -> ElRef<E> {
    reference.clone()
}

// ------ ElRef ------

/// DOM element reference.
/// You want to use it instead of DOM selectors to get raw DOM elements.
///
/// # Example
///
/// ```rust,no_run
/// #[derive(Default)]
/// struct Model {
///     canvas: ElRef<web_sys::HtmlCanvasElement>,
/// }
///
/// fn view(model: &Model) -> impl View<Msg> {
///     canvas![
///         el_ref(&model.canvas),
///         attrs![
///             At::Width => px(200),
///             At::Height => px(100),
///         ],
///     ]
/// }
///
/// fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
///     orders.after_next_render(|_| Msg::Rendered);
/// // ...
///
/// fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
///     match msg {
///         Msg::Rendered => {
///             let canvas = canvas.get().expect("get canvas element");
///             // ...
///             orders.after_next_render(|_| Msg::Rendered).skip();
///         }
/// // ...
/// ```
#[derive(Clone)]
pub struct ElRef<E> {
    pub shared_node_ws: SharedNodeWs,
    // We need to use `phantom` to remember required element type `E`,
    // so we can use it automatically in `get` method for casting.
    phantom: PhantomData<E>,
}

impl<E: Clone + JsCast> ElRef<E> {
    pub fn new() -> Self {
        Self {
            // We need to use interior mutability
            // to modify `Model` from `view` during VDOM patching.
            shared_node_ws: SharedNodeWs::new(),
            phantom: PhantomData,
        }
    }

    /// Get referenced DOM element.
    ///
    /// It returns `Some(element)` when:
    /// - An associated DOM element has been already attached during render.
    /// - The DOM element is still a part of the current DOM.
    /// - The DOM element has the same type like `ElRef`.
    ///
    /// See `ElRef` for more info.
    pub fn get(&self) -> Option<E> {
        // Has `node_ws` already been assigned by VDOM?
        let node_ws = match self.shared_node_ws.clone_inner() {
            Some(node_ws) => node_ws,
            None => return None,
        };
        // Is `node_ws` in the current DOM?
        if !document().contains(Some(&node_ws)) {
            return None;
        }
        // Try to cast to the chosen element type.
        node_ws.dyn_into::<E>().ok()
    }
}

impl<E> Default for ElRef<E> {
    fn default() -> Self {
        Self {
            shared_node_ws: SharedNodeWs::new(),
            phantom: PhantomData,
        }
    }
}

// ------ SharedNodeWs ------

#[derive(Debug, Default, Clone)]
pub struct SharedNodeWs(Rc<RefCell<Option<web_sys::Node>>>);

impl SharedNodeWs {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    pub fn set(&mut self, node_ws: web_sys::Node) {
        self.0.replace(Some(node_ws));
    }

    pub fn clone_inner(&self) -> Option<web_sys::Node> {
        self.0.borrow().clone()
    }
}