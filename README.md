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
SALT_FOR_HASHING="<your hash>"
```
