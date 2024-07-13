FROM ubuntu:24.04
RUN apt-get update && apt-get install -y libheif-dev && rm -rf /var/lib/apt/lists/*
ADD target/release/framelabs-s3-server /bin/framelabs-s3-server
WORKDIR /app
CMD /bin/framelabs-s3-server