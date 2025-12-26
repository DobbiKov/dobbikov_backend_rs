const apiBase = localStorage.getItem('apiBase') || 'http://127.0.0.1:3000';
const token = localStorage.getItem('authToken');
const userMeta = document.getElementById('userMeta');
const apiMeta = document.getElementById('apiMeta');
const statusEl = document.getElementById('status');

const refreshBtn = document.getElementById('refreshBtn');
const logoutBtn = document.getElementById('logoutBtn');

const sectionForm = document.getElementById('sectionForm');
const subsectionForm = document.getElementById('subsectionForm');
const noteForm = document.getElementById('noteForm');

const subsectionSection = document.getElementById('subsectionSection');
const noteParent = document.getElementById('noteParent');
const noteParentSelect = document.getElementById('noteParentSelect');
const sectionsList = document.getElementById('sectionsList');

const state = {
  sections: [],
  subsections: [],
  notes: [],
};

if (!token) {
  window.location.href = 'login.html';
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

function render() {
  sectionsList.innerHTML = '';
  const sections = sortByPosition(state.sections);
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

    actions.append(saveBtn, deleteBtn, upBtn, downBtn);

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

      subActions.append(subSave, subDelete, subUp, subDown);

      const notesList = document.createElement('div');
      notesList.className = 'list';
      const notes = sortByPosition(notesBySubsection.get(subsection.id) || []);

      notes.forEach((note, noteIndex) => {
        notesList.appendChild(buildNoteItem(note, noteIndex, notes));
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

    card.append(title, editInput, actions);
    if (subsections.length) {
      const subHeader = document.createElement('div');
      subHeader.className = 'tag';
      subHeader.textContent = 'Subsections';
      card.appendChild(subHeader);
      card.appendChild(subsectionList);
    }
    if (sectionNoteItems.length) {
      const noteHeader = document.createElement('div');
      noteHeader.className = 'tag';
      noteHeader.textContent = 'Notes';
      card.appendChild(noteHeader);
      card.appendChild(sectionNotes);
    }

    sectionsList.appendChild(card);
  });

  buildSelectOptions(subsectionSection, sections, (item) => item.title);
  const parentItems = noteParent.value === 'section' ? sections : state.subsections;
  buildSelectOptions(noteParentSelect, parentItems, (item) => item.title);
}

function buildNoteItem(note, noteIndex, list) {
  const noteItem = document.createElement('div');
  noteItem.className = 'item';

  const noteTitle = document.createElement('div');
  noteTitle.className = 'item-title';
  noteTitle.textContent = `${note.name} (pos ${note.position})`;

  const noteInputs = document.createElement('div');
  noteInputs.className = 'inline-inputs';

  const nameInput = document.createElement('input');
  nameInput.value = note.name;

  const urlInput = document.createElement('input');
  urlInput.value = note.url;

  noteInputs.append(nameInput, urlInput);

  const noteActions = document.createElement('div');
  noteActions.className = 'actions';

  const noteSave = document.createElement('button');
  noteSave.textContent = 'Save';
  noteSave.addEventListener('click', async () => {
    await updateNote(note.id, nameInput.value.trim(), urlInput.value.trim());
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
  noteItem.append(noteTitle, noteInputs, noteActions);
  return noteItem;
}

async function loadAll() {
  clearStatus();
  try {
    const [sections, subsections, notes] = await Promise.all([
      apiFetch('/sections', { method: 'GET' }),
      apiFetch('/subsections', { method: 'GET' }),
      apiFetch('/notes', { method: 'GET' }),
    ]);
    state.sections = sections;
    state.subsections = subsections;
    state.notes = notes;
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

async function updateNote(id, name, url) {
  await apiFetch(`/notes/${id}`, {
    method: 'PUT',
    body: JSON.stringify({ name, url }),
  });
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
  const items = noteParent.value === 'section' ? state.sections : state.subsections;
  buildSelectOptions(noteParentSelect, items, (item) => item.title);
});

noteForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  try {
    const parentId = Number(noteForm.noteParentSelect.value);
    const payload = {
      name: noteForm.noteName.value.trim(),
      url: noteForm.noteUrl.value.trim(),
      section_id: noteParent.value === 'section' ? parentId : null,
      subsection_id: noteParent.value === 'subsection' ? parentId : null,
    };
    await createNote(payload);
    noteForm.reset();
    await loadAll();
  } catch (err) {
    setStatus(err.message || 'Failed to create note');
  }
});

refreshBtn.addEventListener('click', () => loadAll());

logoutBtn.addEventListener('click', () => {
  localStorage.removeItem('authToken');
  localStorage.removeItem('authUser');
  localStorage.removeItem('tokenExpiresAt');
  window.location.href = 'login.html';
});

const user = JSON.parse(localStorage.getItem('authUser') || 'null');
if (user) {
  userMeta.textContent = `Signed in as ${user.username} (${user.is_admin ? 'admin' : 'viewer'})`;
} else {
  userMeta.textContent = 'Signed in';
}
apiMeta.textContent = `API: ${apiBase}`;

loadAll();
