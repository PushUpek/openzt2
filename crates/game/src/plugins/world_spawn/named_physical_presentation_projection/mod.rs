use super::{
    prefab_attachment_independent_rotation::PrefabAttachmentIndependentRotation,
    prefab_authored_attachment_identifier::PrefabAuthoredAttachmentIdentifier,
    prefab_presentation_types::PrefabPresentation, world_membership_types::DefinitionId,
};
use crate::assets::scene_prefab::ScenePrefabAsset;
use crate::assets::world_definitions::{
    world_definition_asset_set_state_and_borrowing_queries::WorldDefinitions,
    world_definition_document_asset_and_demand_loaded_dependency_paths::WorldDefinitionAsset,
};
use bevy::prelude::*;
use openzt2_game_data::{
    world_definitions::world_objects::WorldObjectNamedPhysicalPresentationDefinition, AssetId,
};

#[cfg(test)]
mod node_attachment_tests;

#[derive(Component)]
pub(super) struct NamedPhysicalPresentationsHydrated;

pub(super) fn hydrate_required_named_physical_presentations(
    mut commands: Commands,
    active_definitions: Res<WorldDefinitions>,
    assets: Res<Assets<WorldDefinitionAsset>>,
    owners: Query<(Entity, &DefinitionId), Without<NamedPhysicalPresentationsHydrated>>,
) {
    let Some(definitions) = active_definitions.get(&assets) else {
        return;
    };
    for (owner, definition) in &owners {
        let Some(object) = definitions.find_object(definition.0) else {
            continue;
        };
        let Some(presentations) = object
            .named_physical_presentations
            .iter()
            .filter(|presentation| presentation.required)
            .map(|presentation| {
                let prefab = if presentation.prefab == AssetId::default() {
                    None
                } else {
                    Some(definitions.scene(presentation.prefab)?)
                };
                Some((presentation, prefab))
            })
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        spawn_required_named_physical_presentations(&mut commands, owner, presentations);
    }
}

pub(super) fn spawn_required_named_physical_presentations<'a>(
    commands: &mut Commands,
    owner: Entity,
    presentations: impl IntoIterator<
        Item = (
            &'a WorldObjectNamedPhysicalPresentationDefinition,
            Option<Handle<ScenePrefabAsset>>,
        ),
    >,
) {
    for (presentation, prefab) in presentations {
        // A node-attached binder hangs from a holder that is later reparented
        // onto its node; the binder's own scale stays on the child.
        let parent = presentation.attach_node.as_ref().map_or(owner, |node| {
            let mut holder = commands.spawn((
                Transform::IDENTITY,
                Visibility::Inherited,
                ChildOf(owner),
                PendingNamedPresentationNodeAttachment {
                    owner,
                    node: node.clone(),
                },
            ));
            if !presentation.inherit_attach_rotation {
                holder.insert(PrefabAttachmentIndependentRotation);
            }
            holder.id()
        });
        let mut child = commands.spawn((
            Transform::from_scale(Vec3::splat(presentation.scale)),
            Visibility::Inherited,
            ChildOf(parent),
            PrefabAuthoredAttachmentIdentifier(presentation.name),
        ));
        if let Some(prefab) = prefab {
            child.insert(PrefabPresentation::new(prefab));
        }
    }
    commands
        .entity(owner)
        .insert(NamedPhysicalPresentationsHydrated);
}

/// A binder holder waiting for the node of its owner's model to exist.
#[derive(Component, Debug, Clone)]
pub(super) struct PendingNamedPresentationNodeAttachment {
    owner: Entity,
    node: String,
}

/// Moves node-attached binder holders onto their authored node: a prefab
/// entity with that authored attachment name (static model nodes), or an
/// animated skeleton joint with that name. The node may live in a model that
/// loads later, such as a counter attached to the same building, so a holder
/// keeps waiting rather than giving up.
// ponytail: unresolved holders rescan their owner's tree every frame; add a
// give-up deadline if missing authored nodes ever show up in profiles.
pub(super) fn attach_named_presentations_to_model_nodes(
    mut commands: Commands,
    pending: Query<(Entity, &PendingNamedPresentationNodeAttachment)>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    identifiers: Query<&PrefabAuthoredAttachmentIdentifier>,
    names: Query<&Name>,
) {
    for (holder, attachment) in &pending {
        let node_name = attachment.node.trim();
        let node = AssetId::from_key(&node_name.to_ascii_lowercase());
        let Some(node_entity) = children
            .iter_descendants_depth_first::<Children>(attachment.owner)
            .filter(|candidate| {
                *candidate != holder
                    && !parents
                        .iter_ancestors::<ChildOf>(*candidate)
                        .any(|ancestor| ancestor == holder)
            })
            .find(|candidate| {
                identifiers.get(*candidate).is_ok_and(|id| id.0 == node)
                    || names
                        .get(*candidate)
                        .is_ok_and(|name| name.as_str().eq_ignore_ascii_case(node_name))
            })
        else {
            continue;
        };
        commands
            .entity(holder)
            .insert(ChildOf(node_entity))
            .remove::<PendingNamedPresentationNodeAttachment>();
    }
}
