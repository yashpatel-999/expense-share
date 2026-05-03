# Azure Container Apps Deployment Guide

## Prerequisites

1. **Azure CLI** - [Install Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli)
2. **Docker** - [Install Docker](https://docs.docker.com/get-docker/)
3. **Azure Subscription** - Create one at [portal.azure.com](https://portal.azure.com)
4. **External PostgreSQL** - Already hosted (Neon, AWS RDS, Azure, etc.)

## Step 1: Login to Azure

```bash
az login
```

## Step 2: Create Resource Group

```bash
az group create --name expense-share-rg --location eastus
```

## Step 3: Create Azure Container Registry (ACR)

```bash
az acr create --resource-group expense-share-rg \
  --name expenseshareacr \
  --sku Basic
```

## Step 4: Configure Docker to Push to ACR

```bash
az acr login --name expenseshareacr
```

## Step 5: Build and Push Docker Images

### Backend Image

```bash
# Build the backend image
docker build -t expenseshareacr.azurecr.io/expenses-backend:latest ./expenses_backend

# Push to ACR
docker push expenseshareacr.azurecr.io/expenses-backend:latest
```

### Frontend Image

```bash
# Build the frontend image
docker build -t expenseshareacr.azurecr.io/expenses-frontend:latest ./expenses_frontend

# Push to ACR
docker push expenseshareacr.azurecr.io/expenses-frontend:latest
```

## Step 6: Create Container App Environment

```bash
az containerapp env create --name expense-share-env \
  --resource-group expense-share-rg \
  --location eastus
```

## Step 7: Deploy Backend Container App

Your PostgreSQL database is already hosted externally. Use your existing connection string.

```bash
# Get ACR credentials
ACR_USERNAME=$(az acr credential show --name expenseshareacr --query username -o tsv)
ACR_PASSWORD=$(az acr credential show --name expenseshareacr --query passwords[0].value -o tsv)
 with your external PostgreSQL connection string
az containerapp create \
  --name expenses-backend \
  --resource-group expense-share-rg \
  --environment expense-share-env \
  --image expenseshareacr.azurecr.io/expenses-backend:latest \
  --registry-server expenseshareacr.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --target-port 8080 \
  --ingress external \
  --env-vars \
    DATABASE_URL="postgresql://user:password@your-host:5432/dbname
    DATABASE_URL="postgresql://neondb_owner:<your-password>@expense-share-db.postgres.database.azure.com:5432/neondb?sslmode=require" \
    JWT_SECRET="<your-jwt-secret>" \
    RUST_LOG="info" \
  --cpu 0.5 \
  --memory 1Gi
```

## Step 9: Get Backend URL

```bash
BACKEND_URL=$(az containerapp show --resource-group expense-share-rg \
  --name expenses-backend \
  --query properties.configuration.ingress.fqdn -o tsv)

echo "Backend URL: https://$BACKEND_URL"
```

## Step 10: Deploy Frontend Container App

```bash
# Deploy frontend with backend URL
az containerapp create \
  --name expenses-frontend \
  --resource-group expense-share-rg \
  --environment expense-share-env \
  --image expenseshareacr.azurecr.io/expenses-frontend:latest \
  --registry-server expenseshareacr.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --target-port 80 \
  --ingress external \
  --env-vars \
    API_URL="https://$BACKEND_URL/api" \
  --cpu 0.5 \
  --memory 1Gi
```

## Step 11: Get Frontend URL

```bash
FRONTEND_URL=$(az containerapp show --resource-group expense-share-rg \
  --name expenses-frontend \
  --query properties.configuration.ingress.fqdn -o tsv)

echo "Frontend URL: https://$FRONTEND_URL"
```

## Step 12: Run Database Migrations

If needed, connect to your external PostgreSQL and run migrations from `migrations/20250902144339_create_expenses_table.sql`

## Environment Variables to Update

After deployment, update your frontend environment configuration:

Edit `src/environments/environment.ts`:

```typescript
export const environment = {
  production: true,
  apiUrl: "https://<your-backend-url>/api",
};
```

Then rebuild and push the frontend image:

```bash
docker build -t expenseshareacr.azurecr.io/expenses-frontend:latest ./expenses_frontend
docker push expenseshareacr.azurecr.io/expenses-frontend:latest

# Update the container app
az containerapp update \
  --name expenses-frontend \
  --resource-group expense-share-rg \
  --image expenseshareacr.azurecr.io/expenses-frontend:latest
```

## Monitoring and Logs

### View logs from backend

```bash
az containerapp logs show --name expenses-backend \
  --resource-group expense-share-rg \
  --follow
```

### View logs from frontend

```bash
az containerapp logs show --name expenses-frontend \
  --resource-group expense-share-rg \
  --follow
```

## Cleanup

To delete all resources:

```bash
az group delete --name expense-share-rg --yes
```

## Troubleshooting

### Backend won't connect to database

- Ensure your PostgreSQL firewall rules allow connections from Container Apps
- Check the DATABASE_URL format matches your Azure PostgreSQL connection string
- Verify credentials are correct

### Frontend can't reach backend

- Ensure the API_URL in frontend environment matches the backend FQDN
- Check CORS settings in backend (should allow frontend origin)
- Verify ingress is enabled on both container apps

### Build failures

- Check Docker is properly installed
- Ensure you're logged into ACR: `az acr login --name expenseshareacr`
- Check image tags match the ACR registry name
  connection string is correct
- Verify credentials match your external database
- Check that your database is accessible from Azure
- If using Neon, ensure you have the correct `sslmode=require` in URL
  Create `deploy.sh` for automated deployment:

```bash
#!/bin/bash

RESOURCE_GROUP="expense-share-rg"
LOCATION="eastus"
ACR_NAME="expenseshareacr"

# Create resource group
az group create --name $RESOURCE_GROUP --location $LOCATION

# Create ACR
az acr create --resource-group $RESOURCE_GROUP --name $ACR_NAME --sku Basic

# Login to ACR
az acr login --name $ACR_NAME

# Build and push images
docker build -t $ACR_NAME.azurecr.io/expenses-backend:latest ./expenses_backend
docker push $ACR_NAME.azurecr.io/expenses-backend:latest

docker build -t $ACR_NAME.azurecr.io/expenses-frontend:latest ./expenses_frontend
docker push $ACR_NAME.azurecr.io/expenses-frontend:latest

echo "Images pushed successfully!"
echo "Next: Create Container App Environment and deploy containers using the steps above"
```

Make it executable:

```bash
chmod +x deploy.sh
./deploy.sh
```
