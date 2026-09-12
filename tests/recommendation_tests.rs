use bookfin::recommendation::strategy::select_with_wrap_around;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
struct MockExtract {
    id: Uuid,
    random_key: f64,
}

#[test]
fn test_unseen_extract_exclusion() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    let items = vec![
        MockExtract {
            id: id1,
            random_key: 0.1,
        },
        MockExtract {
            id: id2,
            random_key: 0.5,
        },
        MockExtract {
            id: id3,
            random_key: 0.9,
        },
    ];

    let mut seen = HashSet::new();
    seen.insert(id1);

    // Lorsqu'on cherche un extrait à partir de rnd = 0.0, id1 est ignoré car vu
    let selected = select_with_wrap_around(
        &items,
        |item| seen.contains(&item.id),
        |item| item.random_key,
        0.0,
    );

    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, id2);
}

#[test]
fn test_random_key_wrap_around() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    let items = vec![
        MockExtract {
            id: id1,
            random_key: 0.2,
        },
        MockExtract {
            id: id2,
            random_key: 0.4,
        },
        MockExtract {
            id: id3,
            random_key: 0.6,
        },
    ];

    let seen: HashSet<Uuid> = HashSet::new();

    // Avec rnd = 0.8, aucun extrait n'a une clé >= 0.8.
    // La stratégie doit reboucler (wrap-around) et sélectionner le premier disponible (id1 à 0.2).
    let selected = select_with_wrap_around(
        &items,
        |item| seen.contains(&item.id),
        |item| item.random_key,
        0.8,
    );

    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, id1);
}

#[test]
fn test_wrap_around_with_seen_first_element() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    let items = vec![
        MockExtract {
            id: id1,
            random_key: 0.2,
        },
        MockExtract {
            id: id2,
            random_key: 0.4,
        },
        MockExtract {
            id: id3,
            random_key: 0.6,
        },
    ];

    let mut seen = HashSet::new();
    seen.insert(id1); // Le premier élément à 0.2 a déjà été vu

    // Avec rnd = 0.7, rebouclage : id1 est ignoré car vu, donc id2 (0.4) est choisi
    let selected = select_with_wrap_around(
        &items,
        |item| seen.contains(&item.id),
        |item| item.random_key,
        0.7,
    );

    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, id2);
}

#[test]
fn test_all_seen_returns_none() {
    let id1 = Uuid::new_v4();
    let items = vec![MockExtract {
        id: id1,
        random_key: 0.5,
    }];

    let mut seen = HashSet::new();
    seen.insert(id1);

    let selected = select_with_wrap_around(
        &items,
        |item| seen.contains(&item.id),
        |item| item.random_key,
        0.3,
    );

    assert!(selected.is_none());
}
