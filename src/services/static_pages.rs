use std::collections::{HashMap, HashSet};

use serde_json::json;
use tokio::fs;

const LECTURE_NOTES_TEMPLATE: &str = include_str!("../../templates/lecture_notes.html");
const LECTURE_NOTES_JS_TEMPLATE: &str = include_str!("../../templates/lecture-notes.js");
const NOTE_PAGE_TEMPLATE: &str = include_str!("../../templates/note_page.html");

const SITE_BASE_URL: &str = "https://korotenky.com";

struct GeneratedNote {
    name: String,
    description: String,
    url: String,
    position: u32,
}

struct GeneratedSubsection {
    title: String,
    position: u32,
    notes: Vec<GeneratedNote>,
}

struct GeneratedSection {
    title: String,
    position: u32,
    subsections: Vec<GeneratedSubsection>,
    notes: Vec<GeneratedNote>,
}

struct GeneratedLectureNotes {
    sections: Vec<GeneratedSection>,
}

pub struct GenerationSummary {
    pub note_pages: usize,
}

#[derive(Debug)]
pub enum GenerateStaticPagesError {
    LoadSections,
    LoadSubsections,
    LoadNotes,
    EnvVar(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for GenerateStaticPagesError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "note".to_string()
    } else {
        slug
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn trim_for_meta(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "Lecture note by Yehor Korotenko.".to_string()
    } else {
        trimmed.to_string()
    }
}

fn escape_json_for_html(value: String) -> String {
    value.replace("</", "<\\/")
}

fn note_file_name(section_title: &str, subsection_title: Option<&str>, note_name: &str) -> String {
    let subsection_part = subsection_title.unwrap_or("section-notes");
    format!(
        "{}_{}_{}.html",
        slugify(section_title),
        slugify(subsection_part),
        slugify(note_name)
    )
}

fn hostname_from_url(value: &str) -> String {
    let without_scheme = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
        .unwrap_or(value);
    let host = without_scheme.split('/').next().unwrap_or_default();
    if host.is_empty() {
        "External source".to_string()
    } else {
        host.trim_start_matches("www.").to_string()
    }
}

fn unique_id(prefix: &str, title: &str, index: usize, used: &mut HashSet<String>) -> String {
    let base_slug = slugify(title);
    let base = if base_slug.is_empty() {
        format!("{prefix}-{}", index + 1)
    } else {
        format!("{prefix}-{base_slug}")
    };
    let mut candidate = base.clone();
    let mut suffix = 2usize;

    while used.contains(&candidate) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }

    used.insert(candidate.clone());
    candidate
}

fn display_note_description(
    raw_description: &str,
    section_title: &str,
    subsection_title: Option<&str>,
) -> String {
    let trimmed = raw_description.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    match subsection_title {
        Some(subsection_title) => {
            format!("Lecture note for {subsection_title} in the {section_title} section.")
        }
        None => format!("Lecture note in the {section_title} section."),
    }
}

fn note_page_absolute_url(file_name: &str) -> String {
    format!("{SITE_BASE_URL}/notes_pages/{file_name}")
}

fn lecture_notes_absolute_url() -> String {
    format!("{SITE_BASE_URL}/notes_pages/lecture_notes.html")
}

fn note_item_html(note: &GeneratedNote, position: usize) -> String {
    let meta_text = if note.url.trim().is_empty() {
        "No source URL".to_string()
    } else {
        hostname_from_url(&note.url)
    };

    let link_html = if note.url.trim().is_empty() {
        "<a class=\"note-link\" href=\"#\" aria-disabled=\"true\" tabindex=\"-1\">Link unavailable</a>".to_string()
    } else {
        format!(
            "<a class=\"note-link\" href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" itemprop=\"url\">Open Notes</a>",
            escape_html(&note.url)
        )
    };

    format!(
        "<article class=\"note-card\" itemprop=\"itemListElement\" itemscope itemtype=\"https://schema.org/LearningResource\"><meta itemprop=\"position\" content=\"{position}\" /><meta itemprop=\"educationalUse\" content=\"study reference\" /><meta itemprop=\"isAccessibleForFree\" content=\"true\" /><div class=\"note-content\"><h4 itemprop=\"name\">{}</h4><p class=\"note-meta\">{}</p>{}</div></article>",
        escape_html(&note.name),
        escape_html(&meta_text),
        link_html
    )
}

fn subsection_html(
    subsection_id: &str,
    title: &str,
    notes_html: Vec<String>,
    note_count: usize,
) -> String {
    format!(
        "<div class=\"subsection\" aria-labelledby=\"{subsection_id}-title\"><div class=\"subsection-header\"><h3 class=\"subsection-title\" id=\"{subsection_id}-title\">{}</h3><span class=\"subsection-count\">{} {}</span></div><div class=\"notes-grid\" itemscope itemtype=\"https://schema.org/ItemList\"><meta itemprop=\"numberOfItems\" content=\"{note_count}\" />{}</div></div>",
        escape_html(title),
        note_count,
        if note_count == 1 { "note" } else { "notes" },
        notes_html.join("")
    )
}

fn build_lecture_notes_markup(data: &GeneratedLectureNotes) -> (String, String) {
    let mut toc_items = Vec::new();
    let mut sections_html = Vec::new();
    let mut used_ids = HashSet::new();

    for (section_index, section) in data.sections.iter().enumerate() {
        let section_id = unique_id("section", &section.title, section_index, &mut used_ids);
        toc_items.push(format!(
            "<li><button type=\"button\" data-target-id=\"{}\" aria-current=\"false\">{}</button></li>",
            escape_html(&section_id),
            escape_html(&section.title)
        ));

        let mut section_parts = vec![format!(
            "<p class=\"section-kicker\">Section {}</p><h2 class=\"section-title\" id=\"{}-title\">{}</h2>",
            section_index + 1,
            escape_html(&section_id),
            escape_html(&section.title)
        )];

        if !section.notes.is_empty() {
            let notes_html = section
                .notes
                .iter()
                .enumerate()
                .map(|(index, note)| note_item_html(note, index + 1))
                .collect::<Vec<_>>();
            section_parts.push(subsection_html(
                &format!("{section_id}-section-notes"),
                "Section notes",
                notes_html,
                section.notes.len(),
            ));
        }

        for (subsection_index, subsection) in section.subsections.iter().enumerate() {
            let subsection_id = unique_id(
                "subsection",
                &format!("{} {}", section.title, subsection.title),
                subsection_index,
                &mut used_ids,
            );
            let notes_html = subsection
                .notes
                .iter()
                .enumerate()
                .map(|(index, note)| note_item_html(note, index + 1))
                .collect::<Vec<_>>();

            section_parts.push(subsection_html(
                &subsection_id,
                &subsection.title,
                notes_html,
                subsection.notes.len(),
            ));
        }

        sections_html.push(format!(
            "<section class=\"section\" id=\"{}\" aria-labelledby=\"{}-title\">{}</section>",
            escape_html(&section_id),
            escape_html(&section_id),
            section_parts.join("")
        ));
    }

    (toc_items.join(""), sections_html.join(""))
}

fn lecture_notes_keywords(data: &GeneratedLectureNotes) -> String {
    let mut keywords = vec![
        "lecture notes".to_string(),
        "study materials".to_string(),
        "Yehor Korotenko".to_string(),
        "Universite Paris-Saclay".to_string(),
    ];

    for section in &data.sections {
        if !keywords.iter().any(|keyword| keyword == &section.title) {
            keywords.push(section.title.clone());
        }

        for subsection in &section.subsections {
            if !keywords.iter().any(|keyword| keyword == &subsection.title) {
                keywords.push(subsection.title.clone());
            }
        }
    }

    keywords.join(", ")
}

fn lecture_notes_structured_data(data: &GeneratedLectureNotes) -> String {
    let mut position = 1usize;
    let mut items = Vec::new();

    for section in &data.sections {
        for note in &section.notes {
            let file_name = note_file_name(&section.title, None, &note.name);
            items.push(json!({
                "@type": "ListItem",
                "position": position,
                "url": note_page_absolute_url(&file_name),
                "name": note.name,
                "description": display_note_description(&note.description, &section.title, None),
            }));
            position += 1;
        }

        for subsection in &section.subsections {
            for note in &subsection.notes {
                let file_name = note_file_name(&section.title, Some(&subsection.title), &note.name);
                items.push(json!({
                    "@type": "ListItem",
                    "position": position,
                    "url": note_page_absolute_url(&file_name),
                    "name": note.name,
                    "description": display_note_description(
                        &note.description,
                        &section.title,
                        Some(&subsection.title),
                    ),
                }));
                position += 1;
            }
        }
    }

    escape_json_for_html(
        json!({
            "@context": "https://schema.org",
            "@type": "CollectionPage",
            "name": "Lecture Notes",
            "description": "Lecture notes and study materials by Yehor Korotenko covering mathematics, computer science, and related subjects.",
            "url": lecture_notes_absolute_url(),
            "author": {
                "@type": "Person",
                "name": "Yehor Korotenko",
                "url": SITE_BASE_URL,
            },
            "mainEntity": {
                "@type": "ItemList",
                "name": "Lecture notes by subject",
                "numberOfItems": items.len(),
                "itemListElement": items,
            }
        })
        .to_string(),
    )
}

fn note_keywords(section_title: &str, subsection_title: Option<&str>, note_name: &str) -> String {
    let mut keywords = vec![
        note_name.to_string(),
        section_title.to_string(),
        "lecture note".to_string(),
        "study material".to_string(),
        "pdf".to_string(),
    ];

    if let Some(subsection_title) = subsection_title {
        keywords.push(subsection_title.to_string());
    }

    keywords.push("Yehor Korotenko".to_string());
    keywords.join(", ")
}

pub async fn generate_static_pages(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<GenerationSummary, GenerateStaticPagesError> {
    let sections = crate::services::sections::get_sections(
        pool,
        crate::services::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            limit: None,
        },
    )
    .await
    .map_err(|_| GenerateStaticPagesError::LoadSections)?;

    let subsections = crate::services::subsections::get_subsections(
        pool,
        crate::services::subsections::GetSubsectionsForm {
            id: None,
            title: None,
            position: None,
            section_id: None,
            limit: None,
        },
    )
    .await
    .map_err(|_| GenerateStaticPagesError::LoadSubsections)?;

    let notes = crate::services::lecture_notes::get_notes(
        pool,
        crate::services::lecture_notes::GetNotesForm {
            id: None,
            name: None,
            url: None,
            position: None,
            section_id: None,
            subsection_id: None,
            limit: None,
        },
    )
    .await
    .map_err(|_| GenerateStaticPagesError::LoadNotes)?;

    let mut notes_by_subsection: HashMap<u32, Vec<GeneratedNote>> = HashMap::new();
    let mut notes_by_section: HashMap<u32, Vec<GeneratedNote>> = HashMap::new();

    for note in notes {
        let generated_note = GeneratedNote {
            name: note.name,
            description: note.description,
            url: note.url,
            position: note.position,
        };

        if let Some(subsection_id) = note.subsection_id {
            notes_by_subsection
                .entry(subsection_id)
                .or_default()
                .push(generated_note);
        } else if let Some(section_id) = note.section_id {
            notes_by_section
                .entry(section_id)
                .or_default()
                .push(generated_note);
        }
    }

    let mut subsections_by_section: HashMap<u32, Vec<GeneratedSubsection>> = HashMap::new();

    for subsection in subsections {
        let mut subsection_notes = notes_by_subsection
            .remove(&subsection.id)
            .unwrap_or_default();
        subsection_notes.sort_by_key(|note| note.position);

        let generated_subsection = GeneratedSubsection {
            title: subsection.title,
            position: subsection.position,
            notes: subsection_notes,
        };

        subsections_by_section
            .entry(subsection.section_id)
            .or_default()
            .push(generated_subsection);
    }

    let mut generated_sections: Vec<GeneratedSection> = sections
        .into_iter()
        .map(|section| {
            let mut section_subsections = subsections_by_section
                .remove(&section.id)
                .unwrap_or_default();
            section_subsections.sort_by_key(|subsection| subsection.position);

            let mut section_notes = notes_by_section.remove(&section.id).unwrap_or_default();
            section_notes.sort_by_key(|note| note.position);

            GeneratedSection {
                title: section.title,
                position: section.position,
                subsections: section_subsections,
                notes: section_notes,
            }
        })
        .collect();

    generated_sections.sort_by_key(|section| section.position);

    let data = GeneratedLectureNotes {
        sections: generated_sections,
    };

    let lecture_notes_html_path = std::env::var("LECTURE_NOTES_HTML_PATH")
        .map_err(|_| GenerateStaticPagesError::EnvVar("LECTURE_NOTES_HTML_PATH".to_string()))?;
    let notes_dir = std::env::var("NOTES_DIRECTORY_PATH")
        .map_err(|_| GenerateStaticPagesError::EnvVar("NOTES_DIRECTORY_PATH".to_string()))?;
    let styles_css_path = std::env::var("STYLES_CSS_PATH")
        .map_err(|_| GenerateStaticPagesError::EnvVar("STYLES_CSS_PATH".to_string()))?;

    fs::create_dir_all(&notes_dir).await?;
    if let Some(parent) = std::path::Path::new(&lecture_notes_html_path).parent() {
        fs::create_dir_all(parent).await?;
    }

    let (toc_html, sections_html) = build_lecture_notes_markup(&data);
    let lecture_notes_html = LECTURE_NOTES_TEMPLATE
        .replace(
            "{{LECTURE_NOTES_META_DESCRIPTION}}",
            "Lecture notes and study materials by Yehor Korotenko covering mathematics, computer science, and related subjects.",
        )
        .replace("{{LECTURE_NOTES_META_KEYWORDS}}", &escape_html(&lecture_notes_keywords(&data)))
        .replace(
            "{{LECTURE_NOTES_STRUCTURED_DATA}}",
            &lecture_notes_structured_data(&data),
        )
        .replace("{{TOC_ITEMS}}", &toc_html)
        .replace("{{SECTIONS_HTML}}", &sections_html)
        .replace("{{STYLES_CSS_PATH}}", &styles_css_path);
    fs::write(&lecture_notes_html_path, lecture_notes_html).await?;

    let lecture_notes_js_output = format!("{notes_dir}/lecture-notes.js");
    fs::write(lecture_notes_js_output, LECTURE_NOTES_JS_TEMPLATE).await?;

    let mut note_pages = 0usize;
    for section in &data.sections {
        for note in &section.notes {
            write_note_page(note, &section.title, None, &notes_dir, &styles_css_path).await?;
            note_pages += 1;
        }

        for subsection in &section.subsections {
            for note in &subsection.notes {
                write_note_page(note, &section.title, Some(&subsection.title), &notes_dir, &styles_css_path).await?;
                note_pages += 1;
            }
        }
    }

    Ok(GenerationSummary { note_pages })
}

async fn write_note_page(
    note: &GeneratedNote,
    section_title: &str,
    subsection_title: Option<&str>,
    notes_dir: &str,
    styles_css_path: &str,
) -> Result<(), GenerateStaticPagesError> {
    let file_name = note_file_name(section_title, subsection_title, &note.name);
    let output_path = format!("{notes_dir}/{file_name}");

    let meta_primary = subsection_title.unwrap_or(section_title);
    let meta_secondary = format!("Section: {section_title}");
    let description = display_note_description(&note.description, section_title, subsection_title);
    let structured_data = escape_json_for_html(
        json!({
            "@context": "https://schema.org",
            "@type": "LearningResource",
            "name": note.name,
            "description": description,
            "url": note_page_absolute_url(&file_name),
            "isAccessibleForFree": true,
            "educationalUse": "study reference",
            "author": {
                "@type": "Person",
                "name": "Yehor Korotenko",
                "url": SITE_BASE_URL,
            },
            "about": [
                section_title,
                subsection_title.unwrap_or("section notes"),
            ],
            "sameAs": if note.url.trim().is_empty() {
                serde_json::Value::Null
            } else {
                json!(note.url)
            },
            "encodingFormat": if note.url.trim_end_matches('/').to_lowercase().ends_with(".pdf") {
                serde_json::Value::String("application/pdf".to_string())
            } else {
                serde_json::Value::Null
            },
        })
        .to_string(),
    );

    let html = NOTE_PAGE_TEMPLATE
        .replace("{{NOTE_TITLE}}", &escape_html(&note.name))
        .replace(
            "{{NOTE_DESCRIPTION_META}}",
            &escape_html(&trim_for_meta(&description)),
        )
        .replace("{{NOTE_DESCRIPTION_BODY}}", &escape_html(&description))
        .replace("{{NOTE_URL}}", &escape_html(&note.url))
        .replace("{{NOTE_FILE_NAME}}", &escape_html(&file_name))
        .replace("{{NOTE_META_PRIMARY}}", &escape_html(meta_primary))
        .replace("{{NOTE_META_SECONDARY}}", &escape_html(&meta_secondary))
        .replace(
            "{{NOTE_SECTION_NAME}}",
            &escape_html(subsection_title.unwrap_or(section_title)),
        )
        .replace(
            "{{NOTE_KEYWORDS}}",
            &escape_html(&note_keywords(section_title, subsection_title, &note.name)),
        )
        .replace("{{NOTE_STRUCTURED_DATA}}", &structured_data)
        .replace("{{STYLES_CSS_PATH}}", styles_css_path);

    fs::write(output_path, html).await?;
    Ok(())
}
