/// Logique de sélection avec wrap-around sur random_key.
/// Cette fonction pure permet de tester unitairement le comportement d'évitement des extraits vus
/// et le rebouclage (wrap-around) lorsque random_key >= rnd ne trouve rien.
pub fn select_with_wrap_around<T, F>(
    items: &[T],
    is_seen: impl Fn(&T) -> bool,
    get_key: F,
    rnd: f64,
) -> Option<&T>
where
    F: Fn(&T) -> f64,
{
    // Filtrer les éléments non vus
    let mut unseen: Vec<&T> = items.iter().filter(|item| !is_seen(item)).collect();
    if unseen.is_empty() {
        return None;
    }

    // Trier par random_key croissant
    unseen.sort_by(|a, b| {
        get_key(a)
            .partial_cmp(&get_key(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 1. Chercher le premier >= rnd
    if let Some(&item) = unseen.iter().find(|&&item| get_key(item) >= rnd) {
        return Some(item);
    }

    // 2. Wrap-around : premier élément disponible
    unseen.first().copied()
}
