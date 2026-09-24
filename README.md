# Lecture Notes API + Admin UI

Rust + MySQL backend for managing lecture notes (sections, subsections, notes) with an admin-only API and a static admin UI.

## Features
- Users can register/login and receive bearer tokens (7-day sessions).
- Admin-only access for create/edit/delete/move actions.
- Public read access to notes, sections, subsections, and tags.
- Colored tags that can be assigned to notes (and used to filter them).
- Static HTML/CSS/JS admin console in `web/`.

## Requirements
- Rust (stable)
- MySQL

## Setup
1. Clone the repo and enter the folder.
2. Create `.env` in the project root:

```
DATABASE_URL="mysql://user:password@localhost:3306/lecture_notes"
TESTING_DATABASE_URL="mysql://user:password@localhost:3306/lecture_notes_test"
SALT_FOR_HASHING="<any-random-string>"
SERVER_ADDR="127.0.0.1:3000"
```

3. Build:

```
cargo build -r
```

## Run the API
```
./target/release/backend-rs
```

The server will create missing tables on startup.

## API Authentication
- Register or login to get a token.
- Include the header on admin routes:

```
Authorization: Bearer <token>
```

Tokens last 7 days.

## API Endpoints (summary)
Public:
- `GET /sections`
- `GET /sections/:id`
- `GET /subsections`
- `GET /subsections/:id`
- `GET /notes`
- `GET /notes/:id`
- `GET /tags`
- `GET /tags/:id`
- `POST /users/register`
- `POST /users/login`

Admin-only:
- `POST /sections`
- `PUT /sections/:id`
- `DELETE /sections/:id`
- `POST /sections/move`
- `POST /subsections`
- `PUT /subsections/:id`
- `DELETE /subsections/:id`
- `POST /subsections/move`
- `POST /notes`
- `PUT /notes/:id`
- `DELETE /notes/:id`
- `POST /notes/move`
- `POST /tags`
- `PUT /tags/:id`
- `DELETE /tags/:id`
- `PUT /notes/:id/tags`
- `POST /notes/:id/tags/:tag_id`
- `DELETE /notes/:id/tags/:tag_id`
- `GET /users`

## Tags
Tags have a unique `name` (max 64 characters) and a `color` in hex. Colors are
accepted as `#rrggbb` or `#rgb` and stored as lowercase `#rrggbb`.

```
POST /tags            {"name": "Algebra", "color": "#1e90ff"}  -> 201 {"id": 1, "name": "Algebra", "color": "#1e90ff"}
PUT  /tags/1          {"color": "#ff8800"}                     (name and color are optional)
DELETE /tags/1        removes the tag and unassigns it from every note
```

Assigning tags to notes:

```
PUT    /notes/5/tags    {"tag_ids": [1, 3]}   replace the note's tags ([] clears them)
POST   /notes/5/tags/2                         add one tag
DELETE /notes/5/tags/2                         remove one tag
```

`POST /notes` and `PUT /notes/:id` also accept an optional `tag_ids` array
(on update it replaces the note's tags). Unknown tag ids are rejected with `404`,
a duplicate tag name with `409`, an invalid name or color with `400`.

Every note returned by `GET /notes`, `GET /notes/:id` and `GET /` has a `tags`
array of `{id, name, color}`. Filter notes by tag with `GET /notes?tag_id=1`.

## Using the Admin UI
The UI is static and can be opened directly in a browser.

1. Open `web/register.html` to create an account (set `Admin Role` to admin).
2. Or open `web/login.html` if you already have a user.
3. After login/register, you’ll land in `web/admin.html`.
4. Use the create panels on the left and the list view on the right to edit, move, or delete content.
5. Manage tags (name + color) in the Tags panel; toggle a note's tags on its card and press Save.

Notes:
- The UI stores the API base URL and token in localStorage.
- If you open the HTML files via `file://`, your browser may block API calls. If so, either enable CORS on the backend or serve the `web/` folder with a simple static server.

## Tests
```
make test
```

## Project Structure
- `src/` Rust backend
- `src/db/` SQLx DB layer
- `src/services/` service layer
- `src/routes/` API handlers
- `web/` static admin UI
