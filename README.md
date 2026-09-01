# Rideboard V2

A Proper Rideboard.

## Features

- Blazingly Fast API in Rust
- Dynamic Vue Frontend
- PostgreSQL Database

## Development Setup

### Requirements

- Rust (I use 1.81)
- npm
- Ideally docker to spin up a database to test on

### Steps

#### Database Setup (Docker)

`cd $PROJECT_ROOT`

`docker run -it --rm -p 5432:5432 -v ./src/migrations:/docker-entrypoint-initdb.d -e POSTGRES_USER=rideboard -e POSTGRES_DATABASE=rideboard -e POSTGRES_PASSWORD=supersecurepassword postgres`

#### Setup .env

1. Copy `.env.example` as `.env`

2. Fill in data fields.
  - For `DATABASE_URL`, format is `postgresql://USERNAME:PASSWORD@HOST/DATABASE`, example for just running the docker compose: `postgresql://rideboard:supersecurepassword@localhost:5432/rideboard`
  - Contact an RTP for CSH Auth Credentials for these fields: `CSH_CLIENT_ID`, `CSH_CLIENT_SECRET`, `CSH_AUTH_URL`, `CSH_TOKEN_URL`, `CSH_USERINFO_URL`.
  - Create a local set of keys for the Google Auth, fill out GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET. See [this guide](https://developers.google.com/identity/sign-in/web/sign-in) for guidance.
  - `REDIRECT_DOMAIN` is the full protocol and domain for your project. Ex `http://localhost:8080`, `https://rideboard-v2.cs.house`.
  - `HOST` is the host for your project. Ex `localhost`, `rideboard-v2.cs.house`.
  - `PORT` is the port for your project. Ex `8080`.

#### Running the Project

1. Run postgres and redis with docker-compose. `docker compose up -d`

2. Install frontend dependencies. `cd src/frontend; npm install`

2. Build Frontend. `npm run build` or `npm run dev` for development. 

3. Build and run the server binary. `cargo run server`

### Frontend Development Tip

In order to develop the frontend without repeatedly recompiling the backend binary, the vite development server has been configured to proxy to `localhost:8080` for all API requests.

You can access the frontend and make changes at the port shown in the vite CLI. Please note that logging in will redirect you back to port 8080, so make sure to change it back when you log in.
