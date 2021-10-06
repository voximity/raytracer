use rlua::{MetaMethod, UserData};

use crate::{
    math::Vector3,
    object::{self, SceneObject},
};

pub trait LuaSceneObject: SceneObject + UserData + Clone {}

impl UserData for Vector3 {
    fn add_methods<'lua, T>(methods: &mut T)
    where
        T: rlua::UserDataMethods<'lua, Self>,
    {
        methods.add_meta_method(MetaMethod::Add, |_, &a, b: Vector3| Ok(a + b));
        methods.add_meta_method(MetaMethod::Sub, |_, &a, b: Vector3| Ok(a - b));
        methods.add_meta_method(MetaMethod::Mul, |_, &a, b: Vector3| Ok(a * b));
        methods.add_meta_method(MetaMethod::Mul, |_, &a, b: f64| Ok(a * b));
        methods.add_meta_method(MetaMethod::Div, |_, &a, b: Vector3| Ok(a / b));
        methods.add_meta_method(MetaMethod::Div, |_, &a, b: f64| Ok(a / b));
        methods.add_meta_method(MetaMethod::ToString, |_, &a, ()| Ok(format!("{:?}", a)));
        methods.add_method("magnitude", |_, me, ()| Ok(me.magnitude()));
        methods.add_method("normalize", |_, me, ()| Ok(me.normalize()));
        methods.add_method("dot", |_, me, other: Vector3| Ok(me.dot(other)));
        methods.add_method("cross", |_, me, other: Vector3| Ok(me.cross(other)));
    }
}

impl UserData for object::Aabb {
    fn add_methods<'lua, T>(methods: &mut T)
    where
        T: rlua::UserDataMethods<'lua, Self>,
    {
        methods.add_method("pos", |_, me, ()| Ok(me.pos()));
        methods.add_method("size", |_, me, ()| Ok(me.size()));
    }
}

impl LuaSceneObject for object::Aabb {}
