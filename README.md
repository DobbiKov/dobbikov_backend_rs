# Backend for my site dobbikov.com

The project is written completely in rust

## Author
Yehor Korotenko (yehor.korotenko@outlook.com)

## Stack
- rust 
- mysql

## Features
- database management
- user login
- simple roles
- CRUD for lecture notes 

## Setup
1. `git clone https://github.com/DobbiKov/dobbikov_backend_rs.git`
2. `cd dobbikov_backend_rs` 
3. `touch .env` then read [.env section](##env-file)
4. `cargo build`

## env file
```.env
DATABASE_URL="<your database url>"
TESTING_DATABASE_URL="<your testing database url>"
SALT_FOR_HASHING="<your hash>"
```
`TESTING_DATABASE_URL` should a database that can be easily droped without any sensitive data. It's a database for running unit-tests.

## Tests
The tests are divided into feature-tests and db tests. Run `make test` in order to run tests, the tests are ran signle-threaded to make sure the empty state of the database during running tests.
