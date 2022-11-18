FROM rust:slim

RUN apt-get update && apt-get upgrade -y
RUN apt-get update && apt-get install --no-install-recommends -y \
      pkg-config \
      libssl-dev \
      && rm -rf /var/lib/apt/lists/*

ARG BASE_PATH="/usr/src/link-checker"
ARG DUMMY_MAIN_RS="dummy-main.rs"
WORKDIR ${BASE_PATH}

# Gist to not always rebuild dependencies
# Build dependencies and copy src at the end
COPY Cargo.toml .
COPY Cargo.lock .
RUN echo 'fn main() {}' > ${DUMMY_MAIN_RS}
RUN sed -i "s#src/main.rs#${DUMMY_MAIN_RS}#" Cargo.toml
RUN cargo build --release
RUN sed -i "s#${DUMMY_MAIN_RS}#src/main.rs#" Cargo.toml
RUN rm ${DUMMY_MAIN_RS}


# Build real project
COPY src ./src
RUN cargo install --path .

ENTRYPOINT ["link-checker"]

