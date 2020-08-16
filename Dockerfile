FROM rust:1.45

WORKDIR /usr/src/fubuki-oauth-proxy
COPY . .
RUN cargo install --path .

#EXPOSE 3000
CMD [ "fubuki-oauth-proxy", "3000" ]
