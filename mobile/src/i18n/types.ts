export type SupportedAppLanguage = 'fr' | 'en' | 'es' | 'ru' | 'zh' | 'ja';

export interface AppLanguageOption {
  code: SupportedAppLanguage;
  label: string;
  nativeLabel: string;
}

export const SUPPORTED_APP_LANGUAGES: AppLanguageOption[] = [
  { code: 'fr', label: 'Français', nativeLabel: 'Français' },
  { code: 'en', label: 'English', nativeLabel: 'English' },
  { code: 'es', label: 'Espagnol', nativeLabel: 'Español' },
  { code: 'ru', label: 'Russe', nativeLabel: 'Русский' },
  { code: 'zh', label: 'Chinois simplifié', nativeLabel: '简体中文' },
  { code: 'ja', label: 'Japonais', nativeLabel: '日本語' },
];

export interface Translations {
  common: {
    loading: string;
    close: string;
    back: string;
    error: string;
    retry: string;
    soon: string;
    cancel: string;
  };
  identity: {
    title: string;
    continue_without_account: string;
    opening: string;
    create_account_soon: string;
    login_soon: string;
    note: string;
  };
  languages: {
    onboarding_title: string;
    onboarding_desc: string;
    continue_button: string;
    saving: string;
    validation_min_one: string;
  };
  reading: {
    languages_button: string;
    profile_button: string;
    start_from_beginning: string;
    start_from_beginning_starting: string;
    /** Action principale sur une œuvre déjà commencée ; {page} = page de reprise. */
    resume_work: string;
    /** Action secondaire ; toujours confirmée avant de remettre la progression à 1. */
    restart_from_beginning: string;
    restart_confirm_title: string;
    restart_confirm_message: string;
    restart_confirm_action: string;
    read_next_page: string;
    another_random_page: string;
    end_of_edition: string;
    back_to_discovery: string;
    followed_reading: string;
    page_progress: string;
    page_number_single: string;
    like_accessibility: string;
    dislike_accessibility: string;
    revealed_accessibility: string;
    reading_too_fast_hint: string;
    insufficient_scroll_hint: string;
  };
  library: {
    title: string;
    empty_title: string;
    empty_desc: string;
    resume: string;
    books_count_one: string;
    books_count_many: string;
  };
  profile: {
    title: string;
    library: string;
    settings: string;
    anonymous_reader: string;
    account_section: string;
    create_account_soon: string;
    login_soon: string;
    data_stored_locally: string;
  };
  settings: {
    title: string;
    app_language_section: string;
    reading_languages_section: string;
    reading_languages_desc: string;
  };
}
