const hashParams = window.location.hash.startsWith('#')
  ? new URLSearchParams(window.location.hash.slice(1))
  : null;

if (hashParams) {
  const hashToken = hashParams.get('token');
  const hashApiBase = hashParams.get('apiBase');
  if (hashToken) {
    localStorage.setItem('authToken', hashToken);
  }
  if (hashApiBase) {
    localStorage.setItem('apiBase', hashApiBase);
  }
  window.location.hash = '';
}

const apiBase = localStorage.getItem('apiBase') || 'http://127.0.0.1:3000';
const token = localStorage.getItem('authToken');
const userMeta = document.getElementById('userMeta');
const apiMeta = document.getElementById('apiMeta');
const statusEl = document.getElementById('status');

const refreshBtn = document.getElementById('refreshBtn');
const logoutBtn = document.getElementById('logoutBtn');
const createUserBtn = document.getElementById('createUserBtn');
const generatePagesBtn = document.getElementById('generatePagesBtn');

const sectionForm = document.getElementById('sectionForm');
const subsectionForm = document.getElementById('subsectionForm');
const noteForm = document.getElementById('noteForm');
const tagForm = document.getElementById('tagForm');

const subsectionSection = document.getElementById('subsectionSection');
const noteParent = document.getElementById('noteParent');
const noteParentSelect = document.getElementById('noteParentSelect');
const sectionsList = document.getElementById('sectionsList');
const tagsList = document.getElementById('tagsList');
const noteTagPicker = document.getElementById('noteTagPicker');

const state = {
  sections: [],
  subsections: [],
  notes: [],
  tags: [],
};

if (!token) {
  window.location.href = '/login';
}

function setStatus(message) {
  statusEl.textContent = message;
  statusEl.hidden = false;
}

function clearStatus() {
  statusEl.hidden = true;
}

function authHeaders() {
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function apiFetch(path, options = {}) {
  const headers = {
    'Content-Type': 'application/json',
    ...authHeaders(),
    ...(options.headers || {}),
  };

  const res = await fetch(`${apiBase}${path}`, { ...options, headers });
  if (!res.ok) {
    const message = await res.text();
    throw new Error(message || 'Request failed');
  }
  if (res.status === 204) return null;
  return res.json();
}

function sortByPosition(items) {
  return [...items].sort((a, b) => a.position - b.position);
}

function buildSelectOptions(select, items, labelFn) {
  select.innerHTML = '';
  items.forEach((item) => {
    const option = document.createElement('option');
    option.value = item.id;
    option.textContent = labelFn(item);
    select.appendChild(option);
  });
}

// Pick black or white text depending on how light the tag color is.
function tagTextColor(hex) {
  const value = hex.replace('#', '');
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.6 ? '#2b2b2b' : '#ffffff';
}

// Fill `container` with toggleable tag chips; returns a function reading the selected ids.
function buildTagPicker(container, selectedIds = []) {
  container.innerHTML = '';
  const selected = new Set(selectedIds);
  const checkboxes = [];

  if (!state.tags.length) {
    const empty = document.createElement('span');
    empty.className = 'small';
    empty.textContent = 'No tags yet. Create one in the Tags panel.';
    container.appendChild(empty);
  }

  state.tags.forEach((tag) => {
    const chip = document.createElement('label');
    chip.className = 'tag-chip';
    chip.style.setProperty('--tag-color', tag.color);
    chip.style.setProperty('--tag-text', tagTextColor(tag.color));

    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.value = tag.id;
    checkbox.checked = selected.has(tag.id);
    checkboxes.push(checkbox);

    const dot = document.createElement('span');
    dot.className = 'tag-dot';

    const name = document.createElement('span');
    name.textContent = tag.name;

    chip.append(checkbox, dot, name);
    container.appendChild(chip);
  });

  return () => checkboxes.filter((box) => box.checked).map((box) => Number(box.value));
}

function renderTags() {
  tagsList.innerHTML = '';
  state.tags.forEach((tag) => {
    const row = document.createElement('div');
    row.className = 'tag-row';

    const colorInput = document.createElement('input');
    colorInput.type = 'color';
    colorInput.value = tag.color;
    colorInput.title = 'Tag color';

    const nameInput = document.createElement('input');
    nameInput.value = tag.name;
    nameInput.maxLength = 64;

    const saveBtn = document.createElement('button');
    saveBtn.textContent = 'Save';
    saveBtn.addEventListener('click', async () => {
      try {
        await updateTag(tag.id, nameInput.value.trim(), colorInput.value);
      } catch (err) {
        setStatus(err.message || 'Failed to update tag');
      }
    });

    const deleteBtn = document.createElement('button');
    deleteBtn.className = 'secondary';
    deleteBtn.textContent = 'Delete';
    deleteBtn.addEventListener('click', async () => {
      if (!window.confirm(`Delete tag "${tag.name}"? It will be removed from all notes.`)) {
        return;
      }
      try {
        await deleteTag(tag.id);
      } catch (err) {
        setStatus(err.message || 'Failed to delete tag');
      }
    });

    row.append(colorInput, nameInput, saveBtn, deleteBtn);
    tagsList.appendChild(row);
  });
}

function render() {
  renderTags();
  buildTagPicker(noteTagPicker);
  sectionsList.innerHTML = '';
  const sections = sortByPosition(state.sections);
  const sectionTitles = new Map(sections.map((section) => [section.id, section.title]));
  const subsectionsBySection = new Map();
  const notesBySubsection = new Map();
  const notesBySection = new Map();

  state.subsections.forEach((sub) => {
    if (!subsectionsBySection.has(sub.section_id)) {
      subsectionsBySection.set(sub.section_id, []);
    }
    subsectionsBySection.get(sub.section_id).push(sub);
  });

  state.notes.forEach((note) => {
    if (note.subsection_id) {
      if (!notesBySubsection.has(note.subsection_id)) {
        notesBySubsection.set(note.subsection_id, []);
      }
      notesBySubsection.get(note.subsection_id).push(note);
    } else if (note.section_id) {
      if (!notesBySection.has(note.section_id)) {
        notesBySection.set(note.section_id, []);
      }
      notesBySection.get(note.section_id).push(note);
    }
  });

  sections.forEach((section, index) => {
    const card = document.createElement('div');
    card.className = 'item';

    const title = document.createElement('div');
    title.className = 'item-title';
    title.textContent = `${section.title} (pos ${section.position})`;

    const editInput = document.createElement('input');
    editInput.value = section.title;
    editInput.placeholder = 'Edit title';

    const actions = document.createElement('div');
    actions.className = 'actions';

    const saveBtn = document.createElement('button');
    saveBtn.textContent = 'Save';
    saveBtn.addEventListener('click', async () => {
      await updateSection(section.id, editInput.value.trim());
    });

    const deleteBtn = document.createElement('button');
    deleteBtn.className = 'secondary';
    deleteBtn.textContent = 'Delete';
    deleteBtn.addEventListener('click', async () => {
      await deleteSection(section.id);
    });

    const upBtn = document.createElement('button');
    upBtn.className = 'ghost';
    upBtn.textContent = 'Move up';
    upBtn.disabled = index === 0;
    upBtn.addEventListener('click', async () => {
      await moveSection(section.id, sections[index - 1].id);
    });

    const downBtn = document.createElement('button');
    downBtn.className = 'ghost';
    downBtn.textContent = 'Move down';
    downBtn.disabled = index === sections.length - 1;
    downBtn.addEventListener('click', async () => {
      await moveSection(section.id, sections[index + 1].id);
    });

    const toggleBtn = document.createElement('button');
    toggleBtn.className = 'toggle-btn';
    toggleBtn.textContent = 'Collapse';

    actions.append(toggleBtn, saveBtn, deleteBtn, upBtn, downBtn);

    const subsectionList = document.createElement('div');
    subsectionList.className = 'list';
    const subsections = sortByPosition(subsectionsBySection.get(section.id) || []);

    subsections.forEach((subsection, subIndex) => {
      const subItem = document.createElement('div');
      subItem.className = 'item';

      const subTitle = document.createElement('div');
      subTitle.className = 'item-title';
      subTitle.textContent = `${subsection.title} (pos ${subsection.position})`;

      const subEdit = document.createElement('input');
      subEdit.value = subsection.title;

      const subActions = document.createElement('div');
      subActions.className = 'actions';

      const subSave = document.createElement('button');
      subSave.textContent = 'Save';
      subSave.addEventListener('click', async () => {
        await updateSubsection(subsection.id, subEdit.value.trim());
      });

      const subDelete = document.createElement('button');
      subDelete.className = 'secondary';
      subDelete.textContent = 'Delete';
      subDelete.addEventListener('click', async () => {
        await deleteSubsection(subsection.id);
      });

      const subUp = document.createElement('button');
      subUp.className = 'ghost';
      subUp.textContent = 'Move up';
      subUp.disabled = subIndex === 0;
      subUp.addEventListener('click', async () => {
        await moveSubsection(subsection.id, subsections[subIndex - 1].id);
      });

      const subDown = document.createElement('button');
      subDown.className = 'ghost';
      subDown.textContent = 'Move down';
      subDown.disabled = subIndex === subsections.length - 1;
      subDown.addEventListener('click', async () => {
        await moveSubsection(subsection.id, subsections[subIndex + 1].id);
      });

      const subToggle = document.createElement('button');
      subToggle.className = 'toggle-btn';
      subToggle.textContent = 'Collapse';

      subActions.append(subToggle, subSave, subDelete, subUp, subDown);

      const notesList = document.createElement('div');
      notesList.className = 'list';
      const notes = sortByPosition(notesBySubsection.get(subsection.id) || []);

      notes.forEach((note, noteIndex) => {
        notesList.appendChild(buildNoteItem(note, noteIndex, notes));
      });

      subToggle.addEventListener('click', () => {
        const isCollapsed = notesList.classList.toggle('collapsed');
        subToggle.textContent = isCollapsed ? 'Expand' : 'Collapse';
      });

      subItem.append(subTitle, subEdit, subActions, notesList);
      subsectionList.appendChild(subItem);
    });

    const sectionNotes = document.createElement('div');
    sectionNotes.className = 'list';
    const sectionNoteItems = sortByPosition(notesBySection.get(section.id) || []);
    sectionNoteItems.forEach((note, noteIndex) => {
      sectionNotes.appendChild(buildNoteItem(note, noteIndex, sectionNoteItems));
    });

    const sectionBody = document.createElement('div');
    sectionBody.className = 'section-body';

    card.append(title, editInput, actions);
    if (subsections.length) {
      const subHeader = document.createElement('div');
      subHeader.className = 'tag';
      subHeader.textContent = 'Subsections';
      sectionBody.appendChild(subHeader);
      sectionBody.appendChild(subsectionList);
    }
    if (sectionNoteItems.length) {
      const noteHeader = document.createElement('div');
      noteHeader.className = 'tag';
      noteHeader.textContent = 'Notes';
      sectionBody.appendChild(noteHeader);
      sectionBody.appendChild(sectionNotes);
    }

    card.appendChild(sectionBody);
    toggleBtn.addEventListener('click', () => {
      const isCollapsed = sectionBody.classList.toggle('collapsed');
      toggleBtn.textContent = isCollapsed ? 'Expand' : 'Collapse';
    });

    sectionsList.appendChild(card);
  });

  buildSelectOptions(subsectionSection, sections, (item) => item.title);
  const parentItems = state.subsections;
  const labelFn = (item) => {
    const sectionTitle = sectionTitles.get(item.section_id) || 'Unknown section';
    return `${sectionTitle} · ${item.title}`;
  };
  buildSelectOptions(noteParentSelect, parentItems, labelFn);
}

function buildNoteItem(note, noteIndex, list) {
  const noteItem = document.createElement('div');
  noteItem.className = 'item note-item';

  const noteSummary = document.createElement('div');
  noteSummary.className = 'note-summary';

  const noteTitle = document.createElement('div');
  noteTitle.className = 'item-title';
  noteTitle.textContent = `${note.name} (pos ${note.position})`;

  noteSummary.append(noteTitle);

  const noteInputs = document.createElement('div');
  noteInputs.className = 'note-fields';

  const nameField = document.createElement('label');
  nameField.className = 'field-stack';

  const nameLabel = document.createElement('span');
  nameLabel.className = 'field-label';
  nameLabel.textContent = 'Name';

  const nameInput = document.createElement('input');
  nameInput.value = note.name;
  nameInput.placeholder = 'Note title';
  nameField.append(nameLabel, nameInput);

  const urlField = document.createElement('label');
  urlField.className = 'field-stack';

  const urlLabel = document.createElement('span');
  urlLabel.className = 'field-label field-label-url';
  urlLabel.textContent = 'URL';

  const urlInput = document.createElement('input');
  urlInput.type = 'url';
  urlInput.value = note.url;
  urlInput.placeholder = 'https://example.com/resource';
  urlInput.className = 'url-input';
  urlField.append(urlLabel, urlInput);

  const descriptionField = document.createElement('label');
  descriptionField.className = 'field-stack field-span-full';

  const descriptionLabel = document.createElement('span');
  descriptionLabel.className = 'field-label field-label-description';
  descriptionLabel.textContent = 'Description';

  const descriptionInput = document.createElement('textarea');
  descriptionInput.rows = 4;
  descriptionInput.value = note.description || '';
  descriptionInput.placeholder = 'Short explanation shown with the note';
  descriptionField.append(descriptionLabel, descriptionInput);

  const tagsField = document.createElement('div');
  tagsField.className = 'field-stack field-span-full';

  const tagsLabel = document.createElement('span');
  tagsLabel.className = 'field-label';
  tagsLabel.textContent = 'Tags';

  const tagsPicker = document.createElement('div');
  tagsPicker.className = 'tag-picker';
  const getSelectedTagIds = buildTagPicker(
    tagsPicker,
    (note.tags || []).map((tag) => tag.id)
  );
  tagsField.append(tagsLabel, tagsPicker);

  noteInputs.append(nameField, urlField, descriptionField, tagsField);

  const noteActions = document.createElement('div');
  noteActions.className = 'actions';

  const noteSave = document.createElement('button');
  noteSave.textContent = 'Save';
  noteSave.addEventListener('click', async () => {
    try {
      await updateNote(
        note.id,
        nameInput.value.trim(),
        descriptionInput.value.trim(),
        urlInput.value.trim(),
        getSelectedTagIds()
      );
    } catch (err) {
      setStatus(err.message || 'Failed to update note');
    }
  });

  const noteDelete = document.createElement('button');
  noteDelete.className = 'secondary';
  noteDelete.textContent = 'Delete';
  noteDelete.addEventListener('click', async () => {
    await deleteNote(note.id);
  });

  const noteUp = document.createElement('button');
  noteUp.className = 'ghost';
  noteUp.textContent = 'Move up';
  noteUp.disabled = noteIndex === 0;
  noteUp.addEventListener('click', async () => {
    await moveNote(note.id, list[noteIndex - 1].id);
  });

  const noteDown = document.createElement('button');
  noteDown.className = 'ghost';
  noteDown.textContent = 'Move down';
  noteDown.disabled = noteIndex === list.length - 1;
  noteDown.addEventListener('click', async () => {
    await moveNote(note.id, list[noteIndex + 1].id);
  });

  const link = document.createElement('a');
  link.href = note.url;
  link.className = 'link';
  link.target = '_blank';
  link.rel = 'noopener';
  link.textContent = 'Open';

  noteActions.append(noteSave, noteDelete, noteUp, noteDown, link);
  noteItem.append(noteSummary, noteInputs, noteActions);
  return noteItem;
}

async function loadAll() {
  clearStatus();
  try {
    const [sections, subsections, notes, tags] = await Promise.all([
      apiFetch('/sections', { method: 'GET' }),
      apiFetch('/subsections', { method: 'GET' }),
      apiFetch('/notes', { method: 'GET' }),
      apiFetch('/tags', { method: 'GET' }),
    ]);
    state.sections = sections;
    state.subsections = subsections;
    state.notes = notes;
    state.tags = tags;
    render();
  } catch (err) {
    setStatus(err.message || 'Failed to load data');
  }
}

async function createSection(title) {
  await apiFetch('/sections', {
    method: 'POST',
    body: JSON.stringify({ title }),
  });
}

async function updateSection(id, title) {
  await apiFetch(`/sections/${id}`, {
    method: 'PUT',
    body: JSON.stringify({ title }),
  });
  await loadAll();
}

async function deleteSection(id) {
  await apiFetch(`/sections/${id}`, { method: 'DELETE' });
  await loadAll();
}

async function moveSection(firstId, secondId) {
  await apiFetch('/sections/move', {
    method: 'POST',
    body: JSON.stringify({ first_id: firstId, second_id: secondId }),
  });
  await loadAll();
}

async function createSubsection(title, sectionId) {
  await apiFetch('/subsections', {
    method: 'POST',
    body: JSON.stringify({ title, section_id: Number(sectionId) }),
  });
}

async function updateSubsection(id, title) {
  await apiFetch(`/subsections/${id}`, {
    method: 'PUT',
    body: JSON.stringify({ title }),
  });
  await loadAll();
}

async function deleteSubsection(id) {
  await apiFetch(`/subsections/${id}`, { method: 'DELETE' });
  await loadAll();
}

async function moveSubsection(firstId, secondId) {
  await apiFetch('/subsections/move', {
    method: 'POST',
    body: JSON.stringify({ first_id: firstId, second_id: secondId }),
  });
  await loadAll();
}

async function createNote(payload) {
  await apiFetch('/notes', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

async function updateNote(id, name, description, url, tagIds) {
  await apiFetch(`/notes/${id}`, {
    method: 'PUT',
    body: JSON.stringify({ name, description, url, tag_ids: tagIds }),
  });
  await loadAll();
}

async function createTag(name, color) {
  await apiFetch('/tags', {
    method: 'POST',
    body: JSON.stringify({ name, color }),
  });
}

async function updateTag(id, name, color) {
  await apiFetch(`/tags/${id}`, {
    method: 'PUT',
    body: JSON.stringify({ name, color }),
  });
  await loadAll();
}

async function deleteTag(id) {
  await apiFetch(`/tags/${id}`, { method: 'DELETE' });
  await loadAll();
}

async function deleteNote(id) {
  await apiFetch(`/notes/${id}`, { method: 'DELETE' });
  await loadAll();
}

async function moveNote(firstId, secondId) {
  await apiFetch('/notes/move', {
    method: 'POST',
    body: JSON.stringify({ first_id: firstId, second_id: secondId }),
  });
  await loadAll();
}

async function generatePages() {
  const result = await apiFetch('/pages/generate', {
    method: 'POST',
  });
  setStatus(result.message || 'Pages generated');
}

sectionForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  try {
    await createSection(sectionForm.sectionTitle.value.trim());
    sectionForm.reset();
    await loadAll();
  } catch (err) {
    setStatus(err.message || 'Failed to create section');
  }
});

subsectionForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  try {
    await createSubsection(
      subsectionForm.subsectionTitle.value.trim(),
      subsectionForm.subsectionSection.value
    );
    subsectionForm.reset();
    await loadAll();
  } catch (err) {
    setStatus(err.message || 'Failed to create subsection');
  }
});

noteParent.addEventListener('change', () => {
  const sectionTitles = new Map(state.sections.map((section) => [section.id, section.title]));
  const labelFn = (item) => {
    const sectionTitle = sectionTitles.get(item.section_id) || 'Unknown section';
    return `${sectionTitle} · ${item.title}`;
  };
  buildSelectOptions(noteParentSelect, state.subsections, labelFn);
});

noteForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  try {
    const parentId = Number(noteForm.noteParentSelect.value);
    const tagIds = [...noteTagPicker.querySelectorAll('input:checked')].map((box) =>
      Number(box.value)
    );
    const payload = {
      name: noteForm.noteName.value.trim(),
      description: noteForm.noteDescription.value.trim(),
      url: noteForm.noteUrl.value.trim(),
      section_id: null,
      subsection_id: parentId,
      tag_ids: tagIds,
    };
    await createNote(payload);
    noteForm.reset();
    await loadAll();
  } catch (err) {
    setStatus(err.message || 'Failed to create note');
  }
});

tagForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  try {
    await createTag(tagForm.tagName.value.trim(), tagForm.tagColor.value);
    tagForm.tagName.value = '';
    await loadAll();
  } catch (err) {
    setStatus(err.message || 'Failed to create tag');
  }
});

refreshBtn.addEventListener('click', () => loadAll());
generatePagesBtn.addEventListener('click', async () => {
  try {
    await generatePages();
  } catch (err) {
    setStatus(err.message || 'Failed to generate pages');
  }
});
createUserBtn.addEventListener('click', () => {
  window.location.href = '/admin/create-user';
});

logoutBtn.addEventListener('click', () => {
  localStorage.removeItem('authToken');
  localStorage.removeItem('authUser');
  localStorage.removeItem('tokenExpiresAt');
  window.location.href = '/login';
});

const user = JSON.parse(localStorage.getItem('authUser') || 'null');
if (user) {
  userMeta.textContent = `Signed in as ${user.username} (${user.is_admin ? 'admin' : 'viewer'})`;
} else {
  userMeta.textContent = 'Signed in';
}
apiMeta.textContent = `API: ${apiBase}`;

loadAll();
