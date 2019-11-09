# hpkgserve

"I hope you like [Rust](http://rust-lang.org) and [HTTP Redirects](https://en.wikipedia.org/wiki/URL_redirection#HTTP_status_codes_3xx)" -- Alex v.

# Config

**Required:**
  * S3_ENDPOINT
  * S3_BUCKET
  * S3_KEY
  * S3_SECRET

**Optional:**
  * CACHE_TTL - How often to scan s3 buckets for latest versions (default 30)
  * S3_REGION - Bucket region (default "us-east-1")
  * S3_PREFIX - Prefix within bucket to repos with no leading / (default "", ex: "myrepos/")
  * PUBLIC_URL - Public URL of object storage. (default: https + S3_ENDPOINT + S3_BUCKET)

# Bucket Format
A common bucket data format is assumed inline with Haiku's standard needs:

  * (architecture)/(file or directory with hrev)

# Common Redirector
A "current" alias within each architecture folder is automatically routed to the latest artifact hrev.

# License

Released under the terms of the MIT license.
