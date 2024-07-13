# framelabs-s3-server
A S3 based image provider for the framelabs picture frame

# Environment Variables

| name | cardinality | description |
|------|-------------|-------------|
| BUCKET_NAME | required | name of the bucket |
| PREFIX | optional | prefix for the s3-location |
| SECRET | optional | secret for request authentication (given as query param ?secret={value} |
| REGION | optinal | AWS region, default to eu-central-1 |
| ACCESS_KEY | required | AWS access key |
| SECRET_KEY | required | AWS secret key |

# deployment docker

# deployment systemd

1) build with cargo

```bash
cargo build -r
```

2) systemd service

```text
[Unit]
Description=framelabs-s3-server

[Service]
Environment="ACCESS_KEY=AKIA..."
Environment="SECRET_KEY=YQOp..."
Environment="BUCKET_NAME=photos-bucket"
Environment="PREFIX=by-year/"
ExecStart=/bin/framelabs-s3-server

[Install]
WantedBy=multi-user.target
```