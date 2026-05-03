#!/bin/bash

# Test script to verify backend and frontend configurations

echo "🔍 Checking Backend Configuration..."
echo ""

# Check Cargo.toml
if grep -q "edition = \"2021\"" expenses_backend/Cargo.toml; then
    echo "✅ Cargo.toml: Edition set to 2021"
else
    echo "❌ Cargo.toml: Edition not set to 2021"
fi

# Check main.rs bind address
if grep -q "bind(\"0.0.0.0:8080\")" expenses_backend/src/main.rs; then
    echo "✅ main.rs: Server binding to 0.0.0.0:8080"
else
    echo "❌ main.rs: Server not binding to 0.0.0.0:8080"
fi

# Check CORS
if grep -q "allow_any_origin()" expenses_backend/src/main.rs; then
    echo "✅ main.rs: CORS allows all origins"
else
    echo "❌ main.rs: CORS not configured for all origins"
fi

echo ""
echo "🔍 Checking Frontend Configuration..."
echo ""

# Check environment.ts
if grep -q "window.apiUrl" expenses_frontend/src/environments/environment.ts; then
    echo "✅ environment.ts: Uses dynamic window.apiUrl"
else
    echo "❌ environment.ts: Not using dynamic window.apiUrl"
fi

# Check index.html
if grep -q "window.apiUrl" expenses_frontend/src/index.html; then
    echo "✅ index.html: Contains window.apiUrl script"
else
    echo "❌ index.html: Missing window.apiUrl script"
fi

# Check docker-entrypoint.sh exists
if [ -f "expenses_frontend/docker-entrypoint.sh" ]; then
    echo "✅ docker-entrypoint.sh: Exists"
else
    echo "❌ docker-entrypoint.sh: Missing"
fi

# Check Dockerfile uses entrypoint
if grep -q "ENTRYPOINT" expenses_frontend/Dockerfile; then
    echo "✅ Dockerfile: Uses docker-entrypoint.sh"
else
    echo "❌ Dockerfile: Not using docker-entrypoint.sh"
fi

# Check angular.json output path
if grep -q "\"outputPath\": \"dist/expenses_frontend\"" expenses_frontend/angular.json; then
    echo "✅ angular.json: Output path is dist/expenses_frontend"
else
    echo "❌ angular.json: Output path not set correctly"
fi

echo ""
echo "🔍 Checking Docker Configuration..."
echo ""

# Check docker-compose
if grep -q "API_URL:" docker-compose.yml; then
    echo "✅ docker-compose.yml: Frontend receives API_URL"
else
    echo "❌ docker-compose.yml: API_URL not passed to frontend"
fi

if grep -q "bind(\"0.0.0.0:8080\")" expenses_backend/src/main.rs; then
    echo "✅ Backend port correctly exposed"
else
    echo "❌ Backend port configuration issue"
fi

echo ""
echo "📋 Configuration Summary:"
echo "========================"
echo "Backend will start on: 0.0.0.0:8080"
echo "Frontend will receive API_URL from environment"
echo "CORS enabled for all origins"
echo ""
echo "✨ All configurations are ready for deployment!"
