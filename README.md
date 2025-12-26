# Lecture Notes API + Admin UI

Rust + MySQL backend for managing lecture notes (sections, subsections, notes) with an admin-only API and a static admin UI.

## Features
- Users can register/login and receive bearer tokens (7-day sessions).
- Admin-only access for create/edit/delete/move actions.
- Public read access to notes, sections, and subsections.
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
cargo build
```

## Run the API
```
cargo run
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
- `GET /users`

## Using the Admin UI
The UI is static and can be opened directly in a browser.

1. Open `web/register.html` to create an account (set `Admin Role` to admin).
2. Or open `web/login.html` if you already have a user.
3. After login/register, you’ll land in `web/admin.html`.
4. Use the create panels on the left and the list view on the right to edit, move, or delete content.

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
