#!/bin/bash
set -euo pipefail

awslocal s3api head-bucket --bucket videos 2>/dev/null || awslocal s3 mb s3://videos

awslocal s3api put-bucket-cors --bucket videos --cors-configuration '{
  "CORSRules": [
    {
      "AllowedOrigins": ["*"],
      "AllowedMethods": ["GET", "PUT", "POST", "HEAD", "DELETE"],
      "AllowedHeaders": ["*"],
      "ExposeHeaders": ["ETag", "etag", "x-amz-request-id"],
      "MaxAgeSeconds": 3000
    }
  ]
}'
