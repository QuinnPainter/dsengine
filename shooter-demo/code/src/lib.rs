#![no_std]
extern crate alloc;

use sandstone::{Script, ScriptContext};
use sandstone::fixed::types::*;
use sandstone::hierarchy::HierarchyPoolTrait;

const SPEED: I20F12 = I20F12::lit("1.5");
const SHOOT_COOLDOWN_RELOAD: u32 = 10;

#[derive(Default)]
pub struct PlayerScript {
    shoot_cooldown: u32,
}

sandstone::register_script!(PlayerScript, 1);
impl Script for PlayerScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);
        let keys = sandstone::ironds::input::read_keys();
        if keys.contains(sandstone::ironds::input::Buttons::UP) {
            node.transform.y -= SPEED;
        }
        if keys.contains(sandstone::ironds::input::Buttons::DOWN) {
            node.transform.y += SPEED;
        }
        if keys.contains(sandstone::ironds::input::Buttons::LEFT) {
            node.transform.x -= SPEED;
        }
        if keys.contains(sandstone::ironds::input::Buttons::RIGHT) {
            node.transform.x += SPEED;
        }
        if keys.contains(sandstone::ironds::input::Buttons::A) && self.shoot_cooldown > SHOOT_COOLDOWN_RELOAD {
            let mut transform = node.transform;
            transform.x += I20F12::lit("12"); // center

            let handle = context.hierarchy.spawn_prefab("Bullet", context.hierarchy.root);
            let bullet = context.hierarchy.borrow_mut(handle);
            bullet.transform = transform;
            self.shoot_cooldown = 0;
        }
        self.shoot_cooldown += 1;
    }
}

const BULLET_SPEED: I20F12 = I20F12::lit("5");

#[derive(Default)]
pub struct BulletScript {
}

sandstone::register_script!(BulletScript, 2);
impl Script for BulletScript {
    fn start(&mut self, _context: &mut ScriptContext) {
    }

    fn update(&mut self, context: &mut ScriptContext) {
        let node = context.hierarchy.borrow_mut(context.handle);
        node.transform.y -= BULLET_SPEED;

        let node = context.hierarchy.borrow(context.handle);
        let child = context.hierarchy.borrow(node.child_handle.unwrap());

        let collider_handle = if let sandstone::node::NodeExtensionHandle::RectCollider(n) = child.node_extension {
            n
        } else {
            panic!("");
        };
        let mut died = false;
        let collider = context.hierarchy.borrow(collider_handle);
        for intersecting_node_handle in collider.intersect_list.iter() {
            sandstone::ironds::nocash::print(&context.hierarchy.borrow(*intersecting_node_handle).name);
            if context.hierarchy.borrow(*intersecting_node_handle).name.contains("Enemy") {
                sandstone::ironds::nocash::print("died");
                died = true;
            }
        }
        if died {
            context.hierarchy.destroy_node(context.handle);
        }
    }
}
