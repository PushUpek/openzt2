use bevy::prelude::*;
use openzt2_game_data::species::LifeStage;

use openzt2_zoo_state::animal_lifecycle::types::{Animal, AnimalLifeStage, Parents, Pregnancy};

use super::verification_journey_state_types::{
    VerificationFactCollectionState, VerificationJourneyReport, VerificationJourneySession,
};

/// Pregnant animals, and young animals with recorded parents (#97).
pub(super) fn collect_animal_reproduction_verification_facts(
    session: Res<VerificationJourneySession>,
    mut report: ResMut<VerificationJourneyReport>,
    animals: Query<(Has<Pregnancy>, &AnimalLifeStage, Option<&Parents>), With<Animal>>,
) {
    if session.fact_collection != VerificationFactCollectionState::Collected {
        return;
    }
    let count = |count: usize| f64::from(u32::try_from(count).unwrap_or(u32::MAX));
    let young = |(_, stage, _): &(bool, &AnimalLifeStage, Option<&Parents>)| {
        matches!(stage.0, LifeStage::Juvenile | LifeStage::Young)
    };
    let facts = [
        (
            "pregnant_animals",
            count(animals.iter().filter(|(pregnant, _, _)| *pregnant).count()),
        ),
        ("young_animals", count(animals.iter().filter(young).count())),
        (
            "young_animals_with_mother",
            count(
                animals
                    .iter()
                    .filter(young)
                    .filter(|(_, _, parents)| {
                        parents.is_some_and(|parents| parents.mother.is_some())
                    })
                    .count(),
            ),
        ),
    ];
    for (name, value) in facts {
        report.current_facts.insert(name.to_owned(), value);
    }
}
