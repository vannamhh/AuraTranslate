//! Proofread scan instruction (AD-68): prompt assembly, resolution and trace.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::promptset::{
    prompt_set_create, prompt_set_list, prompt_set_proofread_choice_get,
    prompt_set_proofread_choice_set_global, prompt_set_proofread_choice_set_work,
    prompt_set_proofread_delete_consequence, prompt_set_rename,
};
use auratranslate_lib::core::ai::rag::{ProofreadBlock, assemble_proofread_batch_prompt};
use auratranslate_lib::core::promptset::{
    PromptSetError, PromptSetKind, PromptSetTier, ProofreadDeleteConsequence, ProofreadInstruction,
    ProofreadWorkChoice, ProofreadWorkMode, read_proofread_work_choice,
    resolve_proofread_instruction, write_proofread_work_choice,
};
use auratranslate_lib::core::scope::load_global_config;
use auratranslate_lib::core::store::{Store, StoreSpec};

fn with_source_blocks() -> Vec<ProofreadBlock<'static>> {
    vec![
        ProofreadBlock {
            source_text: "你好，世界。\n第二行",
            target_text: "Xin chao the gioi.\nDong thu hai",
        },
        ProofreadBlock {
            source_text: "",
            target_text: "Mot cau khong co nguon {{source_segment}}",
        },
    ]
}

fn without_source_blocks() -> Vec<ProofreadBlock<'static>> {
    vec![
        ProofreadBlock {
            source_text: "  ",
            target_text: "Cau thu nhat.",
        },
        ProofreadBlock {
            source_text: "",
            target_text: "Cau thu hai\nco hai dong.",
        },
    ]
}

const GOLDEN_WITH_SOURCE: &str = include_str!("fixtures/proofread_prompt_with_source.golden.txt");
const GOLDEN_WITHOUT_SOURCE: &str =
    include_str!("fixtures/proofread_prompt_without_source.golden.txt");

#[test]
fn no_instruction_gives_the_golden_prompt_byte_for_byte() {
    assert_eq!(
        assemble_proofread_batch_prompt(&with_source_blocks(), None),
        GOLDEN_WITH_SOURCE
    );
    assert_eq!(
        assemble_proofread_batch_prompt(&without_source_blocks(), None),
        GOLDEN_WITHOUT_SOURCE
    );
}

#[test]
fn a_blank_instruction_gives_the_golden_prompt_byte_for_byte() {
    assert_eq!(
        assemble_proofread_batch_prompt(&with_source_blocks(), Some(" \n\t ")),
        GOLDEN_WITH_SOURCE
    );
}

const FORMAT_LINE_START: &str = "Reply with only a JSON array of";

#[test]
fn the_instruction_stands_before_the_reply_contract_and_the_contract_is_intact() {
    let blocks = with_source_blocks();
    let prompt =
        assemble_proofread_batch_prompt(&blocks, Some("  Only compare with the source.  "));
    let (intro, contract) = GOLDEN_WITH_SOURCE
        .split_once('\n')
        .expect("golden has an intro");
    assert_eq!(
        prompt,
        format!("{intro}\nOnly compare with the source.\n{contract}")
    );
    assert!(
        prompt.find("Only compare with the source.") < prompt.find(FORMAT_LINE_START),
        "{prompt}"
    );
}

fn marker_line_count(prompt: &str, marker: &str) -> usize {
    prompt.lines().filter(|l| l.trim() == marker).count()
}

#[test]
fn an_instruction_line_equal_to_a_scan_marker_is_broken_by_a_zero_width_space() {
    let markers = [
        "<<<SEGMENT 1",
        "SEGMENT>>>",
        "<<<SOURCE",
        "SOURCE>>>",
        "<<<TEXT",
        "TEXT>>>",
    ];
    let blocks = with_source_blocks();
    let clean = assemble_proofread_batch_prompt(&blocks, None);
    for marker in markers {
        let instruction = format!("Before\n{marker}\nAfter");
        let prompt = assemble_proofread_batch_prompt(&blocks, Some(&instruction));
        let lines: Vec<&str> = prompt.lines().collect();
        let at = lines
            .iter()
            .position(|l| l.contains(&marker[..1]) && l.contains('\u{200B}'));
        let at = at.unwrap_or_else(|| panic!("no guarded line for {marker}: {prompt}"));
        assert_eq!(lines[at].replace('\u{200B}', ""), marker);
        for real in markers {
            assert_eq!(
                marker_line_count(&prompt, real),
                marker_line_count(&clean, real),
                "{marker} -> {real}"
            );
        }
    }
}

#[test]
fn an_instruction_marker_in_a_longer_line_or_another_case_is_left_alone() {
    let blocks = without_source_blocks();
    let instruction = "keep <<<TEXT inside a sentence\nand text>>> too";
    let prompt = assemble_proofread_batch_prompt(&blocks, Some(instruction));
    assert!(prompt.contains(instruction), "{prompt}");
}

#[test]
fn a_marker_placeholder_in_the_instruction_is_sent_verbatim() {
    let blocks = without_source_blocks();
    let prompt = assemble_proofread_batch_prompt(&blocks, Some("See {{source_segment}} here"));
    assert!(prompt.contains("See {{source_segment}} here"), "{prompt}");
}

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct Env {
    global: Store,
    open: OpenWork,
    dirs: Vec<PathBuf>,
}

impl Drop for Env {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

fn env(tag: &str) -> Env {
    let mut dirs = Vec::new();
    let mut dir = |suffix: &str| {
        let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!(
            "auratranslate-proofread-instruction-{}-{tag}-{suffix}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).expect("temp dir");
        dirs.push(d.clone());
        d
    };
    let global_dir = dir("global");
    let work_dir = dir("work");
    let global = Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("global.db");
    let open =
        create_work_from_text(&work_dir, tag, "en", "", "Mot cau.".to_owned()).expect("work");
    Env { global, open, dirs }
}

impl Env {
    fn create(&self, tier: PromptSetTier, kind: PromptSetKind, name: &str, body: &str) -> i64 {
        prompt_set_create(Some(&self.global), Some(&self.open), tier, kind, name, body)
            .expect("create set")
            .id
    }

    fn proofread(&self, tier: PromptSetTier, name: &str, body: &str) -> i64 {
        self.create(tier, PromptSetKind::Proofread, name, body)
    }

    fn choose_global(&self, name: Option<&str>) {
        prompt_set_proofread_choice_set_global(Some(&self.global), name).expect("global choice");
    }

    fn choose_work(&self, name: Option<&str>, off: bool) {
        prompt_set_proofread_choice_set_work(Some(&self.open), name, off).expect("work choice");
    }

    fn resolve(&self) -> Result<Option<ProofreadInstruction>, PromptSetError> {
        resolve_proofread_instruction(&self.open.scope, &self.global, Some(&self.open.store))
    }

    fn body(&self) -> Option<String> {
        self.resolve().expect("resolves").map(|i| i.body)
    }

    fn consequence(&self, tier: PromptSetTier, id: i64) -> ProofreadDeleteConsequence {
        prompt_set_proofread_delete_consequence(Some(&self.global), Some(&self.open), tier, id)
            .expect("consequence")
    }
}

#[test]
fn no_choice_at_either_tier_resolves_to_no_instruction() {
    let e = env("none");
    e.proofread(PromptSetTier::Global, "B", "global body");
    assert_eq!(e.resolve().expect("resolves"), None);
}

#[test]
fn the_work_choice_wins_then_the_global_one_and_off_silences_both() {
    let e = env("order");
    e.proofread(PromptSetTier::Global, "A", "body of A");
    e.proofread(PromptSetTier::Global, "B", "body of B");
    e.choose_global(Some("B"));

    assert_eq!(e.body().as_deref(), Some("body of B"));
    let followed = e.resolve().expect("resolves").expect("some");
    assert_eq!(
        (followed.name.as_str(), followed.tier),
        ("B", PromptSetTier::Global)
    );

    e.choose_work(Some("A"), false);
    assert_eq!(e.body().as_deref(), Some("body of A"));

    e.choose_work(None, false);
    assert_eq!(e.body().as_deref(), Some("body of B"));

    e.choose_work(None, true);
    assert_eq!(e.resolve().expect("resolves"), None);
}

#[test]
fn a_work_choice_naming_only_a_global_set_resolves_to_it_and_a_work_set_shadows_the_global_one() {
    let e = env("tiers");
    e.proofread(PromptSetTier::Global, "A", "global A");
    e.choose_work(Some("A"), false);
    let got = e.resolve().expect("resolves").expect("some");
    assert_eq!(
        (got.body.as_str(), got.tier),
        ("global A", PromptSetTier::Global)
    );

    e.proofread(PromptSetTier::Work, "A", "work A");
    let got = e.resolve().expect("resolves").expect("some");
    assert_eq!(
        (got.body.as_str(), got.tier),
        ("work A", PromptSetTier::Work)
    );
}

#[test]
fn a_blank_body_is_no_instruction() {
    let e = env("blank");
    e.proofread(PromptSetTier::Global, "Empty", " \n\t ");
    e.choose_global(Some("Empty"));
    assert_eq!(e.resolve().expect("resolves"), None);
}

#[test]
fn a_chosen_name_that_resolves_nowhere_is_an_error_naming_the_set_and_the_tier_of_the_choice() {
    let e = env("missing");
    e.choose_work(Some("X"), false);
    match e.resolve() {
        Err(PromptSetError::ProofreadSetMissing { name, selected_at }) => {
            assert_eq!((name.as_str(), selected_at), ("X", PromptSetTier::Work));
        }
        other => panic!("expected ProofreadSetMissing, got {other:?}"),
    }

    e.choose_work(None, false);
    e.choose_global(Some("G"));
    match e.resolve() {
        Err(PromptSetError::ProofreadSetMissing { name, selected_at }) => {
            assert_eq!((name.as_str(), selected_at), ("G", PromptSetTier::Global));
        }
        other => panic!("expected ProofreadSetMissing, got {other:?}"),
    }
}

#[test]
fn a_translate_set_with_the_chosen_name_never_stands_in_for_the_proofread_one() {
    let e = env("kinds");
    e.create(
        PromptSetTier::Global,
        PromptSetKind::Translate,
        "A",
        "translate body",
    );
    e.choose_global(Some("A"));
    assert!(matches!(
        e.resolve(),
        Err(PromptSetError::ProofreadSetMissing { .. })
    ));

    e.proofread(PromptSetTier::Global, "A", "proofread body");
    assert_eq!(e.body().as_deref(), Some("proofread body"));
}

#[test]
fn the_translation_path_lists_no_proofread_set() {
    let e = env("kinds-list");
    e.create(
        PromptSetTier::Global,
        PromptSetKind::Translate,
        "T",
        "t {{source_segment}}",
    );
    e.proofread(PromptSetTier::Global, "P", "p");
    e.proofread(PromptSetTier::Work, "PW", "pw");

    let translate =
        prompt_set_list(Some(&e.global), Some(&e.open), PromptSetKind::Translate).expect("list");
    let names: Vec<&str> = translate.sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["T"]);
    let proofread =
        prompt_set_list(Some(&e.global), Some(&e.open), PromptSetKind::Proofread).expect("list");
    let names: Vec<&str> = proofread.sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["P", "PW"]);
}

#[test]
fn renaming_the_set_the_open_work_resolves_to_moves_the_work_choice() {
    let e = env("rename-work");
    let id = e.proofread(PromptSetTier::Work, "A", "body of A");
    e.choose_work(Some("A"), false);

    prompt_set_rename(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Work,
        PromptSetKind::Proofread,
        id,
        "C",
    )
    .expect("rename");

    assert_eq!(
        read_proofread_work_choice(&e.open.store).expect("choice"),
        ProofreadWorkChoice::Named("C".to_owned())
    );
    let got = e.resolve().expect("resolves").expect("some");
    assert_eq!((got.name.as_str(), got.body.as_str()), ("C", "body of A"));
}

#[test]
fn renaming_a_global_set_moves_the_global_choice_and_a_following_work() {
    let e = env("rename-global");
    let id = e.proofread(PromptSetTier::Global, "B", "body of B");
    e.choose_global(Some("B"));

    prompt_set_rename(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        id,
        "D",
    )
    .expect("rename");

    assert_eq!(
        load_global_config(&e.global)
            .expect("config")
            .proofread_prompt_set()
            .as_deref(),
        Some("D")
    );
    let got = e.resolve().expect("resolves").expect("some");
    assert_eq!((got.name.as_str(), got.body.as_str()), ("D", "body of B"));
}

#[test]
fn renaming_a_global_set_a_work_set_shadows_leaves_the_work_choice_and_moves_the_global_one() {
    let e = env("rename-shadowed");
    e.proofread(PromptSetTier::Work, "A", "work A");
    let global_id = e.proofread(PromptSetTier::Global, "A", "global A");
    e.choose_work(Some("A"), false);
    e.choose_global(Some("A"));

    prompt_set_rename(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        global_id,
        "C",
    )
    .expect("rename");

    assert_eq!(
        read_proofread_work_choice(&e.open.store).expect("choice"),
        ProofreadWorkChoice::Named("A".to_owned())
    );
    assert_eq!(
        load_global_config(&e.global)
            .expect("config")
            .proofread_prompt_set()
            .as_deref(),
        Some("C")
    );
    assert_eq!(e.body().as_deref(), Some("work A"));
}

#[test]
fn renaming_a_global_set_to_a_name_the_work_already_has_keeps_the_work_choice() {
    let e = env("rename-onto-work-name");
    e.proofread(PromptSetTier::Work, "C", "work C");
    let global_id = e.proofread(PromptSetTier::Global, "A", "global A");
    e.choose_work(Some("A"), false);
    e.choose_global(Some("A"));

    prompt_set_rename(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        global_id,
        "C",
    )
    .expect("rename");

    assert_eq!(
        read_proofread_work_choice(&e.open.store).expect("choice"),
        ProofreadWorkChoice::Named("A".to_owned())
    );
    assert_eq!(
        load_global_config(&e.global)
            .expect("config")
            .proofread_prompt_set()
            .as_deref(),
        Some("C")
    );
    assert!(matches!(
        e.resolve(),
        Err(PromptSetError::ProofreadSetMissing { .. })
    ));
}

#[test]
fn a_rename_that_fails_leaves_the_choices_alone() {
    let e = env("rename-fails");
    let id = e.proofread(PromptSetTier::Work, "A", "a");
    e.proofread(PromptSetTier::Work, "Taken", "t");
    e.choose_work(Some("A"), false);

    let err = prompt_set_rename(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Work,
        PromptSetKind::Proofread,
        id,
        "Taken",
    )
    .expect_err("name taken");
    assert_eq!(err.code(), "prompt_set.name_taken");
    assert_eq!(
        read_proofread_work_choice(&e.open.store).expect("choice"),
        ProofreadWorkChoice::Named("A".to_owned())
    );
}

#[test]
fn deleting_a_chosen_work_set_reports_who_takes_over() {
    let e = env("delete");
    let work_id = e.proofread(PromptSetTier::Work, "A", "work A");
    e.choose_work(Some("A"), false);
    assert_eq!(
        e.consequence(PromptSetTier::Work, work_id),
        ProofreadDeleteConsequence::SetMissing
    );

    e.proofread(PromptSetTier::Global, "A", "global A");
    assert_eq!(
        e.consequence(PromptSetTier::Work, work_id),
        ProofreadDeleteConsequence::GlobalTakesOver
    );

    let other = e.proofread(PromptSetTier::Work, "Other", "o");
    assert_eq!(
        e.consequence(PromptSetTier::Work, other),
        ProofreadDeleteConsequence::None
    );
}

#[test]
fn deleting_a_global_set_the_scan_resolves_to_reports_it_missing_and_a_shadowing_work_set_is_unaffected()
 {
    let e = env("delete-global");
    let global_id = e.proofread(PromptSetTier::Global, "G", "g");
    e.choose_global(Some("G"));
    assert_eq!(
        e.consequence(PromptSetTier::Global, global_id),
        ProofreadDeleteConsequence::SetMissing
    );

    e.choose_work(None, true);
    assert_eq!(
        e.consequence(PromptSetTier::Global, global_id),
        ProofreadDeleteConsequence::SetMissing,
        "the Global choice is still there even while this Work is off"
    );

    e.choose_work(Some("S"), false);
    let shadowed = e.proofread(PromptSetTier::Global, "S", "global S");
    e.proofread(PromptSetTier::Work, "S", "work S");
    assert_eq!(
        e.consequence(PromptSetTier::Global, shadowed),
        ProofreadDeleteConsequence::None
    );
}

#[test]
fn the_work_choice_writes_both_columns_at_once_and_refuses_a_blank_name() {
    let e = env("write");
    e.choose_work(Some("A"), false);
    e.choose_work(Some("A"), true);
    let row: (Option<String>, i64) = e
        .open
        .store
        .read(|c| {
            c.query_row(
                "SELECT chosen_proofread_prompt_set_name, proofread_instruction_off FROM work WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("row");
    assert_eq!(row, (None, 1), "off wins and the name goes back to NULL");

    for blank in ["", "   "] {
        let err = prompt_set_proofread_choice_set_work(Some(&e.open), Some(blank), false)
            .expect_err("blank refused");
        assert_eq!(err.code(), "prompt_set.invalid_name");
    }
    write_proofread_work_choice(&e.open.store, &ProofreadWorkChoice::FollowGlobal).expect("reset");
    assert_eq!(
        read_proofread_work_choice(&e.open.store).expect("choice"),
        ProofreadWorkChoice::FollowGlobal
    );
}

#[test]
fn a_null_global_choice_removes_the_row() {
    let e = env("global-null");
    e.choose_global(Some("B"));
    assert_eq!(
        load_global_config(&e.global)
            .expect("config")
            .proofread_prompt_set()
            .as_deref(),
        Some("B")
    );
    e.choose_global(None);
    assert_eq!(
        load_global_config(&e.global)
            .expect("config")
            .proofread_prompt_set(),
        None
    );
    let err =
        prompt_set_proofread_choice_set_global(Some(&e.global), Some("  ")).expect_err("blank");
    assert_eq!(err.code(), "prompt_set.invalid_name");
}

fn choices(e: &Env) -> auratranslate_lib::core::promptset::ProofreadChoiceState {
    prompt_set_proofread_choice_get(Some(&e.global), Some(&e.open)).expect("choices")
}

#[test]
fn the_read_command_reports_follow_global_and_no_global_choice_by_default() {
    let e = env("read-default");
    let got = choices(&e);
    assert_eq!(got.global, None);
    let work = got.work.expect("work open");
    assert_eq!(
        (work.mode, work.name, work.missing),
        (ProofreadWorkMode::FollowGlobal, None, false)
    );
}

#[test]
fn the_read_command_flags_a_chosen_name_that_resolves_in_neither_tier() {
    let e = env("read-missing");
    e.proofread(PromptSetTier::Global, "B", "global body");
    e.choose_global(Some("B"));
    e.choose_work(Some("X"), false);

    let got = choices(&e);
    let global = got.global.expect("global choice");
    assert_eq!((global.name.as_str(), global.missing), ("B", false));
    let work = got.work.expect("work open");
    assert_eq!(
        (work.mode, work.name.as_deref(), work.missing),
        (ProofreadWorkMode::Named, Some("X"), true)
    );
    assert!(matches!(
        e.resolve(),
        Err(PromptSetError::ProofreadSetMissing { .. })
    ));
}

#[test]
fn the_read_command_agrees_with_the_scan_after_a_delete_and_a_rename() {
    let e = env("read-agree");
    let id = e.proofread(PromptSetTier::Global, "B", "body");
    e.choose_global(Some("B"));
    assert!(!choices(&e).global.expect("choice").missing);

    prompt_set_rename(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        id,
        "C",
    )
    .expect("rename");
    let global = choices(&e).global.expect("choice");
    assert_eq!((global.name.as_str(), global.missing), ("C", false));

    auratranslate_lib::commands::promptset::prompt_set_delete(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        id,
    )
    .expect("delete");
    let global = choices(&e).global.expect("choice");
    assert_eq!((global.name.as_str(), global.missing), ("C", true));
    assert!(e.resolve().is_err());
}

#[test]
fn a_work_choice_naming_a_global_only_set_is_not_missing_and_off_is_never_missing() {
    let e = env("read-two-tier");
    e.proofread(PromptSetTier::Global, "A", "body");
    e.choose_work(Some("A"), false);
    assert!(!choices(&e).work.expect("work").missing);
    e.choose_work(None, true);
    let work = choices(&e).work.expect("work");
    assert_eq!(
        (work.mode, work.name, work.missing),
        (ProofreadWorkMode::Off, None, false)
    );
}

#[test]
fn the_read_command_without_an_open_work_has_no_work_state() {
    let e = env("read-no-work");
    e.choose_global(Some("B"));
    let got = prompt_set_proofread_choice_get(Some(&e.global), None).expect("choices");
    assert!(got.work.is_none());
    assert!(got.global.expect("global").missing);
}

#[test]
fn a_proofread_body_never_warns_and_lists_no_variables_while_translate_still_does() {
    let e = env("read-warnings");
    let created = prompt_set_create(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        "P",
        "so sanh {{source_segment}} va {{glossary_terms}}",
    )
    .expect("create");
    assert!(created.warnings.unknown_markers.is_empty());
    assert!(!created.warnings.glossary_terms_missing);

    let proofread_list =
        prompt_set_list(Some(&e.global), Some(&e.open), PromptSetKind::Proofread).expect("list");
    assert!(proofread_list.variables.is_empty());

    let translate_list =
        prompt_set_list(Some(&e.global), Some(&e.open), PromptSetKind::Translate).expect("list");
    assert!(!translate_list.variables.is_empty());
    let translated = prompt_set_create(
        Some(&e.global),
        Some(&e.open),
        PromptSetTier::Global,
        PromptSetKind::Translate,
        "T",
        "{{nope}}",
    )
    .expect("create");
    assert!(translated.warnings.glossary_terms_missing);
}
