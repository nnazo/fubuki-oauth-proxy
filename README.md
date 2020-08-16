# fubuki-oauth-proxy

A web server for proxying OAuth2 token exchanges to AniList, allowing you store your client secret on the proxy server instead in your client application.

## Prerequisites
* Rust

## Usage
1. In `Settings.toml` set the `client_secret` to your client secret.
2. Run the server in the docker container using `docker-compose up --build`