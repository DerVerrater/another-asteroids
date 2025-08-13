//! All the resources for the game

use bevy::{
    asset::{Assets, Handle},
    ecs::{
        resource::Resource,
        world::{FromWorld, World},
    },
    math::{
        Vec2,
        primitives::{Circle, Triangle2d},
    },
    prelude::{Deref, DerefMut, Reflect, ReflectResource},
    render::mesh::Mesh,
    sprite::ColorMaterial,
};
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;

use crate::{
    ASTEROID_SMALL_COLOR, BULLET_COLOR, PLAYER_SHIP_COLOR, SHIP_THRUSTER_COLOR_ACTIVE,
    SHIP_THRUSTER_COLOR_INACTIVE, config::WINDOW_SIZE,
};

#[derive(Resource, Debug, Deref, Clone, Copy)]
pub struct Score(pub i32);

impl From<Score> for String {
    fn from(value: Score) -> Self {
        value.to_string()
    }
}

#[derive(InspectorOptions, Reflect, Resource, Debug, Deref, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Lives(pub i32);

impl From<Lives> for String {
    fn from(value: Lives) -> Self {
        value.to_string()
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct WorldSize(Vec2);

impl Default for WorldSize {
    fn default() -> Self {
        WorldSize(Vec2::new(WINDOW_SIZE.x, WINDOW_SIZE.y))
    }
}

#[derive(Resource)]
pub struct GameAssets {
    meshes: [Handle<Mesh>; 5],
    materials: [Handle<ColorMaterial>; 7],
}

impl GameAssets {
    pub fn ship(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[0].clone(), self.materials[0].clone())
    }

    // The thruster mesh is actually just the ship mesh
    pub fn thruster_mesh(&self) -> Handle<Mesh> {
        self.meshes[0].clone()
    }

    // TODO: Look into parameterizing the material
    // A shader uniform should be able to do this, but I don't know how to
    // load those in Bevy.
    pub fn thruster_mat_inactive(&self) -> Handle<ColorMaterial> {
        self.materials[1].clone()
    }

    pub fn thruster_mat_active(&self) -> Handle<ColorMaterial> {
        self.materials[2].clone()
    }

    pub fn asteroid_small(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[1].clone(), self.materials[1].clone())
    }

    pub fn asteroid_medium(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[2].clone(), self.materials[2].clone())
    }

    pub fn asteroid_large(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[3].clone(), self.materials[3].clone())
    }

    pub fn bullet(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[4].clone(), self.materials[6].clone())
    }
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let mut world_meshes = world.resource_mut::<Assets<Mesh>>();
        let meshes = [
            world_meshes.add(Triangle2d::new(
                Vec2::new(0.5, 0.0),
                Vec2::new(-0.5, 0.45),
                Vec2::new(-0.5, -0.45),
            )),
            world_meshes.add(Circle::new(10.0)),
            world_meshes.add(Circle::new(20.0)),
            world_meshes.add(Circle::new(40.0)),
            world_meshes.add(Circle::new(0.2)),
        ];
        let mut world_materials = world.resource_mut::<Assets<ColorMaterial>>();
        let materials = [
            world_materials.add(PLAYER_SHIP_COLOR),
            world_materials.add(SHIP_THRUSTER_COLOR_INACTIVE),
            world_materials.add(SHIP_THRUSTER_COLOR_ACTIVE),
            world_materials.add(ASTEROID_SMALL_COLOR),
            // TODO: asteroid medium and large colors
            world_materials.add(ASTEROID_SMALL_COLOR),
            world_materials.add(ASTEROID_SMALL_COLOR),
            world_materials.add(BULLET_COLOR),
        ];
        GameAssets { meshes, materials }
    }
}
