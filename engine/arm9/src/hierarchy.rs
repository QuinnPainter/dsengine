use core::num::NonZeroU32;
use alloc::{string::String, boxed::Box, vec::Vec};
use crate::{Script, pool::{Pool, Handle}, ScriptContext};

#[derive(Eq, PartialEq, Clone, Copy, Default, Debug)]
pub struct Transform {
    pub x: u32,
    pub y: u32,
}

pub struct NodeScriptData {
    pub type_id: NonZeroU32,
    pub script: Box<dyn Script>
}

pub struct Node {
    pub child_handle: Option<Handle<Node>>, // todo: maybe could be more efficient, could just be Index without Generation
    pub sibling_handle: Option<Handle<Node>>,
    pub name: String,
    pub transform: Transform,
    pub script_data: Option<NodeScriptData>,
    pub enabled: bool,
}

impl Node {
    pub fn cast_script<T>(&self) -> &T
    where T: Script + HasTypeId {
        let s_data = self.script_data.as_ref().expect("Tried to cast_script on an object which has no Script");
        assert_eq!(<T as HasTypeId>::type_id(), s_data.type_id, "Tried to cast_script with mismatching types");
        unsafe { &*(s_data.script.as_ref() as *const dyn Script as *const T) }
    }

    pub fn cast_script_mut<T>(&mut self) -> &mut T
    where T: Script + HasTypeId {
        let s_data = self.script_data.as_mut().expect("Tried to cast_script on an object which has no Script");
        assert_eq!(<T as HasTypeId>::type_id(), s_data.type_id, "Tried to cast_script with mismatching types");
        unsafe { &mut *(s_data.script.as_mut() as *mut dyn Script as *mut T) }
    }
}

pub struct Hierarchy {
    pub root: Handle<Node>,
    object_pool: Pool<Node>,
    to_start_stack: Vec<Handle<Node>>
}

impl Hierarchy {
    pub fn new() -> Self {
        let mut object_pool: Pool<Node> = Pool::new();
        let root = object_pool.add(Node {
            child_handle: None,
            sibling_handle: None,
            name: String::new(),
            transform: Transform::default(),
            script_data: None,
            enabled: true
        });

        Self {
            root,
            object_pool,
            to_start_stack: Vec::new(),
        }
    }

    pub fn temp(&mut self) {
        let mut g = dsengine_common::SavedNodeGraph {nodes: Vec::new()};
        g.nodes.push(dsengine_common::SavedNode {
            child_index: None,
            sibling_index: None,
            name: String::from("derp"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: None,
            enabled: true
        });

        let mut tmp: Vec<Handle<Node>> = Vec::new();
        for node in g.nodes {
            tmp.push(self.object_pool.add(Node {
                child_handle: None,
                sibling_handle: None,
                name: node.name,
                transform: Transform { x: node.transform.x, y: node.transform.y },
                script_data: match node.script_type_id {
                    Some(id) => Some(NodeScriptData {
                        type_id: id,
                        script: run_script_factory(id)
                    }),
                    None => None
                },
                enabled: node.enabled
            }));
        }
        //let a = dsengine_common::do_serialize(&g);
        //ironds::nocash::print(&alloc::format!("{:?}", a).to_string());
    }

    pub fn add(&mut self, item: Node, parent: Handle<Node>) {
        let handle = self.object_pool.add(item);
        let parent_obj = self.object_pool.borrow_mut(parent);
        self.object_pool.borrow_mut(handle).sibling_handle = parent_obj.child_handle.replace(handle);

        self.to_start_stack.push(handle);
    }

    #[inline]
    #[must_use]
    pub fn borrow2(&self, handle: Handle<Node>) -> &Node {
        self.object_pool.borrow(handle)
    }

    // todo: recursive search?
    // could have fast path for situation where search root is graph root node
    // as we can iterate over vec sequentially instead of following the tree

    #[must_use]
    pub fn find_by_name(&mut self, search_root: Handle<Node>, name: &str) -> Option<Handle<Node>> {
        self.find(search_root, |x| x.name == name)
    }

    #[must_use]
    pub fn find_by_script_type<T>(&mut self, search_root: Handle<Node>) -> Option<Handle<Node>>
    where T: Script + HasTypeId {
        self.find(search_root, |x| {
            if x.script_data.is_some() {
                return x.script_data.as_ref().unwrap().type_id == <T as HasTypeId>::type_id();
            }
            false
        })
    }

    #[must_use]
    pub fn find<P>(&mut self, search_root: Handle<Node>, mut predicate: P) -> Option<Handle<Node>>
    where P: FnMut(&Node) -> bool, {
        let mut cur_node_handle = self.object_pool.borrow(search_root).child_handle?;
        loop {
            let cur_node = self.object_pool.borrow(cur_node_handle);
            if predicate(cur_node) {
                return Some(cur_node_handle);
            }
            cur_node_handle = cur_node.sibling_handle?;
        }
    }

    pub(crate) fn run_pending_script_starts(&mut self) {
        while self.run_one_pending_script_start() {}
    }

    pub(crate) fn run_one_pending_script_start(&mut self) -> bool{
        if let Some(handle) = self.to_start_stack.pop() {
            // this could return None if an object was immediately destroyed after creating it
            if let Some((item_ticket, mut item)) = self.object_pool.try_take(handle) {
                if let Some(script_data) = &mut item.script_data {
                    let mut context = ScriptContext { hierarchy: self };
                    script_data.script.start(&mut context);
                }
                self.object_pool.put_back(item_ticket, item);
            }
            return true;
        }
        false
    }

    pub(crate) fn run_script_update(&mut self) {
        for i in 0..self.object_pool.vec_len() {
            if let Some((item_ticket, mut item)) = self.object_pool.try_take_by_index(i) {
                if let Some(script_data) = &mut item.script_data {
                    let mut context = ScriptContext { hierarchy: self };
                    script_data.script.update(&mut context);
                }
                self.object_pool.put_back(item_ticket, item);
            }
        }
    }
}

static mut SCRIPT_FACTORY: Option<fn(NonZeroU32) -> Box<dyn Script>> = None;

pub fn init_script_factory(f: fn(NonZeroU32) -> Box<dyn Script>) {
    unsafe { SCRIPT_FACTORY = Some(f); }
}

#[inline]
#[must_use]
pub (crate) fn run_script_factory(id: NonZeroU32) -> Box<dyn Script> {
    unsafe {
        debug_assert_ne!(SCRIPT_FACTORY, None, "Cannot use Script Factory before initialisation!");
        SCRIPT_FACTORY.unwrap_unchecked()(id)
    }
}

pub trait HasTypeId {
    fn type_id() -> NonZeroU32;
}
