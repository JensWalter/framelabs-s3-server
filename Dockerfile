FROM ubuntu:24.04 AS builder
RUN apt-get update && apt-get install -y libheif-dev zlib1g-dev librust-libheif-sys-dev build-essential curl  && rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
ADD Cargo.toml .
ADD src src
RUN cargo build -r

FROM ubuntu:24.04 AS runner
RUN apt-get update && apt-get install -y libheif-dev ca-certificates && update-ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/framelabs-s3-server /bin/framelabs-s3-server
WORKDIR /app
CMD /bin/framelabs-s3-server