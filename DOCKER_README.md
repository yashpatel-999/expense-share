# Expense Share - Docker & Azure Container Apps

This guide helps you containerize and deploy the Expense Share application to Azure Container Apps.

## Quick Start

### 1. Create Environment File

```bash
# Copy the example and add your PostgreSQL URL
cp .env.example .env
```

Edit `.env` and update the `DATABASE_URL` with your PostgreSQL connection string:

```
DATABASE_URL=postgresql://user:password@your-host:5432/dbname?sslmode=require
JWT_SECRET=your-secret-key
RUST_LOG=info
```

### 2. Test Locally with Docker Compose

```bash
docker-compose up -d
```

This will start:

- Backend API on `http://localhost:8080`
- Frontend on `http://localhost`

Your PostgreSQL database is external - no local database service needed.

### 2. Deploy to Azure

#### Option A: Automated Script (Recommended)

```bash
chmod +x deploy-azure.sh
./deploy-azure.sh
```

#### Option B: Manual Steps

See [AZURE_DEPLOYMENT.md](./AZURE_DEPLOYMENT.md) for step-by-step instructions.

## File Structure

```
expenses_backend/
├── Dockerfile           # Multi-stage build for Rust backend
├── src/
├── migrations/
└── Cargo.toml

expenses_frontend/
├── Dockerfile           # Multi-stage build for Angular frontend
├── nginx.conf          # Nginx configuration for production
└── src/

docker-compose.yml      # Local development setup
AZURE_DEPLOYMENT.md     # Detailed Azure deployment guide
deploy-azure.sh         # Automated deployment script
```

## Docker Images

### Backend Image

- **Base**: `rust:latest` (build) → `debian:bookworm-slim` (runtime)
- **Port**: 8080
- **Size**: Optimized with multi-stage build

### Frontend Image

- **Base**: `node:20-alpine` (build) → `nginx:alpine` (runtime)
- **Port**: 80
- **Features**: Gzip compression, cache busting, security headers

## Key Features

✅ Multi-stage Docker builds for minimal image sizes
✅ Health checks configured
✅ Environment variable management
✅ PostgreSQL integration
✅ JWT authentication ready
✅ CORS configuration
✅ Production-ready Nginx setup
✅ Gzip compression enabled
✅ Static asset caching

## Environment Variables

### Backend

```
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require  # Your external PostgreSQL
JWT_SECRET=your-secret-key
RUST_LOG=info
```

### Frontend

```
API_URL=https://backend-url/api
```

## Scaling in Azure

To adjust resources:

```bash
# Scale backend
az containerapp update --name expenses-backend \
  --resource-group expense-share-rg \
  --cpu 1 --memory 2Gi

# Scale frontend
az containerapp update --name expenses-frontend \
  --resource-group expense-share-rg \
  --cpu 0.5 --memory 1Gi
```

## Monitoring

View logs in real-time:

```bash
# Backend logs
az containerapp logs show --name expenses-backend \
  --resource-group expense-share-rg --follow

# Frontend logs
az containerapp logs show --name expenses-frontend \
  --resource-group expense-share-rg --follow
```

## CI/CD with GitHub Actions

See `.github/workflows/azure-deploy.yml` for automated deployment on push.

## Troubleshooting

### Database Connection Issues

- Verify PostgreSQL firewall allows Container Apps IP
- Check `DATABASE_URL` format
- Ensure database exists and credentials are correct

### Frontend/Backend Communication

- Verify `API_URL` matches backend's public URL
- Check CORS headers in backend
- Ensure both apps are in same Container App Environment

### Image Push Failures

- Run `az acr login --name <registry-name>`
- Check Docker is running
- Verify image tags are correct

## Cost Optimization

- Use **B1ms** tier for PostgreSQL (≈$30/month)
- Use **0.5 CPU, 1GB RAM** for Container Apps (≈$15/month each)
- Set up auto-scaling based on traffic
- Use Azure Storage for static assets (optional)

## Production Checklist

- [ ] Update JWT secret to a strong value
- [ ] Configure database backups
- [ ] Set up monitoring and alerts
- [ ] Enable Application Insights
- [ ] Configure custom domain
- [ ] Set up SSL certificates (Azure handles this)
- [ ] Configure application logging
- [ ] Test disaster recovery

## Support

For issues with Azure services, visit:

- [Azure Container Apps Documentation](https://learn.microsoft.com/azure/container-apps/)
- [Azure Database for PostgreSQL](https://learn.microsoft.com/azure/postgresql/)
- [Azure Container Registry](https://learn.microsoft.com/azure/container-registry/)
