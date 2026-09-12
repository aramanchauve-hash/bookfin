use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use uuid::Uuid;

use crate::application::dtos::{BookMetadataDto, ReactionTypeDto};
use crate::web::server_fns::{get_next_extract, record_reaction};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="fr">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (reveal_data, set_reveal_data) = signal::<Option<BookMetadataDto>>(None);
    let (is_submitting, set_is_submitting) = signal(false);

    // Ressource pour charger l'extrait courant
    let extract_resource = Resource::new(|| (), |_| async move { get_next_extract().await });

    // Fonction pour passer à l'extrait suivant
    let next_extract = move || {
        set_reveal_data.set(None);
        extract_resource.refetch();
    };

    // Action pour envoyer une réaction (Like, Skip, Save)
    let on_react = move |extract_id: Uuid, reaction_type: ReactionTypeDto| {
        set_is_submitting.set(true);
        leptos::task::spawn_local(async move {
            let event_id = Uuid::new_v4();
            let _ = record_reaction(event_id, extract_id, reaction_type).await;
            set_is_submitting.set(false);
            next_extract();
        });
    };

    // Action pour le bouton Reveal
    let on_reveal = move |extract_id: Uuid| {
        set_is_submitting.set(true);
        leptos::task::spawn_local(async move {
            let event_id = Uuid::new_v4();
            if let Ok(res) = record_reaction(event_id, extract_id, ReactionTypeDto::Reveal).await {
                set_reveal_data.set(res.metadata);
            }
            set_is_submitting.set(false);
        });
    };

    view! {
        <Title text="Bookfin - Découverte Littéraire"/>
        <Stylesheet id="leptos" href="/pkg/bookfin.css"/>

        <main class="container">
            <h1 class="header-title">Extrait Littéraire</h1>

            <Suspense fallback=move || view! { <p class="loading-state">"Chargement de l'extrait..."</p> }>
                {move || {
                    extract_resource.get().map(|result| match result {
                        Ok(Some(extract)) => {
                            let ext_id = extract.id;
                            let ext_content = extract.content.clone();

                            view! {
                                <div class="extract-card">
                                    <p class="extract-text">{ext_content}</p>

                                    {move || {
                                        if let Some(ref meta) = reveal_data.get() {
                                            let year_str = meta.publication_year.map(|y| format!(" ({})", y)).unwrap_or_default();
                                            let meta_info = format!("Langue : {}{}", meta.original_language_tag, year_str);
                                            let title = meta.title.clone();
                                            let author = meta.author.clone();
                                            view! {
                                                <div class="reveal-box">
                                                    <h2 class="reveal-title">{title}</h2>
                                                    <p class="reveal-author">{author}</p>
                                                    <p class="reveal-meta">{meta_info}</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span/> }.into_any()
                                        }
                                    }}
                                </div>

                                <div class="actions-container">
                                    {move || {
                                        if reveal_data.get().is_some() {
                                            // Après Reveal : affichage du bouton Next
                                            view! {
                                                <button
                                                    class="btn-next"
                                                    disabled=move || is_submitting.get()
                                                    on:click=move |_| next_extract()
                                                >
                                                    "Next"
                                                </button>
                                            }.into_any()
                                        } else {
                                            // Écran principal : Like, Skip, Save, puis Reveal
                                            view! {
                                                <div class="reaction-buttons">
                                                    <button
                                                        disabled=move || is_submitting.get()
                                                        on:click=move |_| on_react(ext_id, ReactionTypeDto::Like)
                                                    >
                                                        "♡ Like"
                                                    </button>
                                                    <button
                                                        disabled=move || is_submitting.get()
                                                        on:click=move |_| on_react(ext_id, ReactionTypeDto::Skip)
                                                    >
                                                        "× Skip"
                                                    </button>
                                                    <button
                                                        disabled=move || is_submitting.get()
                                                        on:click=move |_| on_react(ext_id, ReactionTypeDto::Save)
                                                    >
                                                        "☆ Save"
                                                    </button>
                                                </div>

                                                <button
                                                    class="btn-reveal"
                                                    disabled=move || is_submitting.get()
                                                    on:click=move |_| on_reveal(ext_id)
                                                >
                                                    "Reveal"
                                                </button>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        }
                        Ok(None) => view! {
                            <div class="empty-state">
                                <p>"Vous avez découvert tous les extraits disponibles pour le moment !"</p>
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="empty-state">
                                <p>"Erreur lors du chargement : " {e.to_string()}</p>
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </main>
    }
}
