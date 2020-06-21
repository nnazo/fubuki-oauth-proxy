# fubuki-oauth-proxy

A web server for proxying OAuth2 token exchanges to AniList, allowing you store your client secret on the proxy server instead in your client application.

## Prerequisites
* Rust

## Usage
1. In `Settings.toml` set the `client_secret` to your client secret.
2. Run the program providing a port to run on as a command line argument, e.g. `cargo run --release 8081`