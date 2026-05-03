#!/bin/sh
set -e

# Replace API_URL placeholder in index.html with actual environment variable
if [ -n "$API_URL" ]; then
  sed -i "s|__API_URL__|$API_URL|g" /usr/share/nginx/html/index.html
else
  sed -i "s|__API_URL__||g" /usr/share/nginx/html/index.html
fi

# Start Nginx
exec nginx -g "daemon off;"
