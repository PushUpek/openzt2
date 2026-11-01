use bevy::prelude::*;

use openzt2_zoo_state::animal_health::types::Disease;
use openzt2_zoo_state::animal_lifecycle::types::Animal;

use super::verification_journey_state_types::{
    VerificationFactCollectionState, VerificationJourneyReport, VerificationJourneySession,
};

/// Sick animals and cures. Cures are counted every frame because an animal
/// can fall ill and be cured between journey snapshots.
pub(super) fn collect_animal_health_verification_facts(
    session: Res<VerificationJourneySession>,
    mut report: ResMut<VerificationJourneyReport>,
    diseased: Query<(), (With<Animal>, With<Disease>)>,
    animals: Query<(), With<Animal>>,
    mut cured: RemovedComponents<Disease>,
    mut cures: Local<u32>,
) {
    *cures += cured
        .read()
        .filter(|entity| animals.contains(*entity))
        .count() as u32;
    if session.fact_collection != VerificationFactCollectionState::Collected {
        return;
    }
    for (name, value) in [
        ("diseased_animals", diseased.iter().count() as f64),
        ("animals_cured", f64::from(*cures)),
    ] {
        report.current_facts.insert(name.to_owned(), value);
    }
}
