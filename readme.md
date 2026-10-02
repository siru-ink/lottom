# Lottom

![Static Badge](https://img.shields.io/badge/rust-2024-orange)
![Static Badge](https://img.shields.io/badge/release-0.3.0-orange)
![Static Badge](https://img.shields.io/badge/self--hosted-yes-blue)
![Static Badge](https://img.shields.io/badge/license-Source--Available-green)

... a simple web grocery list tracker with multi user and cross user lists
support.

## Self-Hosting Installation

See the included `docker-compose.yml` file for more information on running
Lottom as a docker container.

### Expected Environmental Variables

| Variable Name | Purpose |
| --- | --- |
| COOKIEKEY | Used to encrypte the cookies stored in the clients browser. Must be a hex encoded 256-bit key. Can be created using something like `openssl rand -hex 32`. |
| POSTGRES_... | These are the standard environmental variables required for running a PostgreSQL database. More information can be found [here](https://hub.docker.com/_/postgres/).
