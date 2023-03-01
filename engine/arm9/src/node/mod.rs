use core::num::NonZeroU32;
use alloc::{string::String, boxed::Box};
use crate::{Script, pool::Handle, hierarchy::HasTypeId};

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
    pub parent_handle: Option<Handle<Node>>, // should only be None on root node
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